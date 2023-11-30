use crate::monitor::monitoring_data::{MonitorVec, MonitoringData};
use crate::sqlite_db::dbutils::SQLiteDB;
use crate::system_info::SystemInfo;
use chrono::Utc;
use log::{info, warn};
use std::path::PathBuf;
use tokio::time::{sleep, Duration};

pub async fn task_periodic_get_os_data() {
    let table_names: Vec<&str> = vec![
        "DataPer10Second",
        "DataPer1Minute",
        "DataPer5Minute",
        "DataPer1Hour",
    ];

    let sqlite_db = SQLiteDB::new(PathBuf::from("my_linux.db")).unwrap();
    for t_name in table_names.iter() {
        if let Ok(()) = sqlite_db.crate_table(t_name) {
            info!("{} 数据表加载完成", t_name);
        } else {
            warn!("{} 数据表加载失败", t_name);
        }
    }

    let mut system_info_instance = SystemInfo::new();
    let mut monitor_10s_data = MonitorVec::new(10);
    let mut monitor_1min_data = MonitorVec::new(6);
    let mut monitor_5min_data = MonitorVec::new(5);
    let mut monitor_1h_data = MonitorVec::new(12);

    loop {
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

        sleep(Duration::from_secs(1)).await;
    }
}

fn insert_data(sqlite: &SQLiteDB, data: &MonitoringData, table_name: &str) {
    if let Ok(()) = sqlite.insert_data(table_name, data.clone()) {
        // info!("插入数据表 {} 成功", table_name);
    } else {
        // warn!("插入数据表 {} 失败", table_name);
    }
}
