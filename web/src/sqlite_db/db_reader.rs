use crate::monitor::monitoring_data::MonitorVec;
use rusqlite::{Connection, Result};
use std::collections::VecDeque;
use std::path::PathBuf;

#[derive(Clone)]
pub struct ReadData {
    pub time_vec: Vec<i64>,
    pub monitor_vec: MonitorVec,
}
pub struct SQLiteReader {
    conn: Connection,
}

pub struct ReadConfig {
    metric_name: Vec<String>,
    start_time: i64,
    stop_time: i64,
    period: i64,
}

impl ReadData {
    pub fn new() -> Self {
        Self {
            time_vec: Vec::new(),
            monitor_vec: MonitorVec::new(0),
        }
    }
}

impl ReadConfig {
    pub fn new(metric_name: Vec<String>, start_time: i64, stop_time: i64, period: i64) -> Self {
        Self {
            metric_name,
            start_time,
            stop_time,
            period,
        }
    }
}

impl SQLiteReader {
    pub fn new(db_path: PathBuf) -> Result<Self> {
        Ok(Self {
            conn: Connection::open(db_path)?,
        })
    }
    pub fn read_only(
        &mut self,
        read_config: &ReadConfig,
    ) -> Result<ReadData, Box<dyn std::error::Error>> {
        let table_name: String = Self::select_table(read_config.period);
        let sql: String = Self::build_sql_query(read_config, table_name);
        let mut stmt: rusqlite::Statement<'_> = self.conn.prepare(&sql)?;
        let mut result_set = stmt.query([])?;

        let mut result = ReadData::new();

        while let Some(row) = result_set.next()? {
            result.monitor_vec.len += 1;
            result.time_vec.push(row.get(0)?);
            result.monitor_vec.load_1.push_back(row.get(1)?);
            result.monitor_vec.load_5.push_back(row.get(2)?);
            result.monitor_vec.load_15.push_back(row.get(3)?);
            result.monitor_vec.cpu_usage.push_back(row.get(4)?);
            result.monitor_vec.memory_used.push_back(row.get(5)?);
            result.monitor_vec.memory_free.push_back(row.get(6)?);
            result.monitor_vec.swap_used.push_back(row.get(7)?);
            result.monitor_vec.swap_free.push_back(row.get(8)?);
            result.monitor_vec.disk_used.push_back(row.get(9)?);
            result.monitor_vec.disk_read.push_back(row.get(10)?);
            result.monitor_vec.disk_write.push_back(row.get(11)?);
            result.monitor_vec.network_rx.push_back(row.get(12)?);
            result.monitor_vec.network_tx.push_back(row.get(13)?);
        }
        Ok(result)
    }
    pub fn read_with_fill(
        &mut self,
        read_config: &ReadConfig,
    ) -> Result<ReadData, Box<dyn std::error::Error>> {
        let mut result = Self::read_only(self, read_config).unwrap();

        Self::audit_data(&mut result, &read_config);

        Ok(result)
    }

    fn select_table(period: i64) -> String {
        match period {
            10 => "DataPer10Second".to_string(),
            60 => "DataPer1Minute".to_string(),
            300 => "DataPer5Minute".to_string(),
            3600 => "DataPer1Hour".to_string(),
            86400 => "DataPer1Hour".to_string(),
            _ => "DataPer10Second".to_string(),
        }
    }

    fn build_sql_query(read_config: &ReadConfig, table_name: String) -> String {
        format!(
            "SELECT * FROM {} WHERE timestamp >= {} AND timestamp < {};",
            table_name, read_config.start_time, read_config.stop_time
        )
    }
    fn audit_data(read_data: &mut ReadData, read_config: &ReadConfig) {
        let req_len = (read_config.stop_time - read_config.start_time) / read_config.period;
        let get_len = read_data.monitor_vec.len as i64;
        let temp = read_config.stop_time / read_config.period * read_config.period;
        let stop_time = temp.clone();
        let start_time = stop_time - read_config.period * (req_len - 1);
        let full_time_vec: Vec<i64> = (start_time..=stop_time)
            .step_by(read_config.period as usize)
            .collect();

        if get_len < req_len {
            let miss_vec = Self::find_miss(&full_time_vec, &read_data.time_vec);
            read_data.time_vec.clone_from(&full_time_vec);
            Self::fill_data(&mut read_data.monitor_vec.load_1, &miss_vec, 0.0);
            Self::fill_data(&mut read_data.monitor_vec.load_5, &miss_vec, 0.0);
            Self::fill_data(&mut read_data.monitor_vec.load_15, &miss_vec, 0.0);
            Self::fill_data(&mut read_data.monitor_vec.cpu_usage, &miss_vec, 0.0);
            Self::fill_data(&mut read_data.monitor_vec.memory_used, &miss_vec, 0);
            Self::fill_data(&mut read_data.monitor_vec.memory_free, &miss_vec, 0);
            Self::fill_data(&mut read_data.monitor_vec.swap_used, &miss_vec, 0);
            Self::fill_data(&mut read_data.monitor_vec.swap_free, &miss_vec, 0);
            Self::fill_data(&mut read_data.monitor_vec.swap_free, &miss_vec, 0);
            Self::fill_data(&mut read_data.monitor_vec.disk_used, &miss_vec, 0);
            Self::fill_data(&mut read_data.monitor_vec.disk_read, &miss_vec, 0);
            Self::fill_data(&mut read_data.monitor_vec.disk_write, &miss_vec, 0);
            Self::fill_data(&mut read_data.monitor_vec.network_rx, &miss_vec, 0);
            Self::fill_data(&mut read_data.monitor_vec.network_tx, &miss_vec, 0);
        }
    }

    fn find_miss(full_times_vec: &Vec<i64>, get_times_vec: &Vec<i64>) -> VecDeque<usize> {
        let mut miss_vec: VecDeque<usize> = VecDeque::new();
        let mut index_get = 0;
        let get_times_vec_len = get_times_vec.len();

        for (i, value) in full_times_vec.iter().enumerate() {
            if get_times_vec_len != 0
                && index_get < get_times_vec_len
                && get_times_vec[index_get] == *value
            {
                index_get += 1;
            } else {
                miss_vec.push_back(i);
            }
        }
        miss_vec
    }

    fn fill_data<T>(vector_data: &mut VecDeque<T>, miss_vec: &VecDeque<usize>, default_data: T)
    where
        T: Copy,
    {
        for index in miss_vec.iter() {
            vector_data.insert(*index, default_data);
        }
    }
}
