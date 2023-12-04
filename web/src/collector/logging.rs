use crate::monitor::monitoring_data::{MonitorVec, MonitoringData};
use crate::sqlite_db::db_reader::{ReadConfig, SQLiteReader};
use crate::sqlite_db::dbutils::SQLiteDB;
use crate::system_info::SystemInfo;
use chrono::Utc;
use log::{info, warn};
use std::path::PathBuf;
use tokio::time::{sleep_until, Duration, Instant};

pub async fn task_periodic_get_os_data() {
    const READ_CONFIG_10S_INTERVAL: i64 = 10;
    const READ_CONFIG_1MIN_INTERVAL: i64 = 60;
    const READ_CONFIG_5MIN_INTERVAL: i64 = 300;

    let db_path = PathBuf::from("my_linux.db");

    let sqlite_db = SQLiteDB::new(db_path.clone()).unwrap();

    let table_names: Vec<&str> = vec![
        "DataPer10Second",
        "DataPer1Minute",
        "DataPer5Minute",
        "DataPer1Hour",
    ];

    for t_name in table_names.iter() {
        if let Ok(()) = sqlite_db.crate_table(t_name) {
            info!("{} load success", t_name);
        } else {
            warn!("{} load failed", t_name);
        }
    }

    let mut reader = SQLiteReader::new(db_path.clone()).unwrap();
    let read_timestamp = Utc::now().timestamp();
    let read_config_1min = ReadConfig::new(
        vec!["all".to_string()],
        read_timestamp - 60 * 1,
        read_timestamp,
        READ_CONFIG_10S_INTERVAL,
    );
    let read_config_5min = ReadConfig::new(
        vec!["all".to_string()],
        read_timestamp - 60 * 5,
        read_timestamp,
        READ_CONFIG_1MIN_INTERVAL,
    );
    let read_config_1hour = ReadConfig::new(
        vec!["all".to_string()],
        read_timestamp - 60 * 5 * 12,
        read_timestamp,
        READ_CONFIG_5MIN_INTERVAL,
    );
    let load_1min_data = reader
        .read_with_fill(&read_config_1min)
        .unwrap()
        .monitor_vec;
    let load_5min_data = reader
        .read_with_fill(&read_config_5min)
        .unwrap()
        .monitor_vec;
    let load_1h_data = reader
        .read_with_fill(&read_config_1hour)
        .unwrap()
        .monitor_vec;
    info!(
        "1min:{}; 5min:{}; 1hour:{};",
        load_1min_data.len, load_5min_data.len, load_1h_data.len
    );
    let mut monitor_10s_data = MonitorVec::new(10);
    let mut monitor_1min_data = load_1min_data;
    let mut monitor_5min_data = load_5min_data;
    let mut monitor_1h_data = load_1h_data;

    let mut last_execution = Instant::now();

    let mut system_info_instance = SystemInfo::new();

    loop {
        last_execution += Duration::from_millis(1000);

        sleep_until(last_execution).await;

        monitor_10s_data.insert(MonitoringData::new(
            system_info_instance.get_less_overview(),
        ));

        let now_timestamp = Utc::now().timestamp();

        if now_timestamp % 10 == 0 {
            let average_10s_data = monitor_10s_data.get_average(now_timestamp.clone());
            insert_data(&sqlite_db, &average_10s_data, table_names[0]);
            monitor_1min_data.insert(average_10s_data);
        }

        if now_timestamp % (10 * 6) == 0 {
            let average_1min_data = monitor_1min_data.get_average(now_timestamp.clone());
            insert_data(&sqlite_db, &average_1min_data, table_names[1]);
            monitor_5min_data.insert(average_1min_data);
        }

        if now_timestamp % (10 * 6 * 5) == 0 {
            let average_5min_data = monitor_5min_data.get_average(now_timestamp.clone());
            insert_data(&sqlite_db, &average_5min_data, table_names[2]);
            monitor_1h_data.insert(average_5min_data);
        }

        if now_timestamp % (10 * 6 * 5 * 12) == 0 {
            let average_1h_data = monitor_1h_data.get_average(now_timestamp.clone());
            insert_data(&sqlite_db, &average_1h_data, table_names[3]);
        }
    }
}

fn insert_data(sqlite: &SQLiteDB, data: &MonitoringData, table_name: &str) {
    if let Ok(()) = sqlite.insert_data(table_name, data.clone()) {
        // info!("Insert Table {} success!", table_name);
    } else {
        // warn!("Insert Table {} failed", table_name);
    }
}
