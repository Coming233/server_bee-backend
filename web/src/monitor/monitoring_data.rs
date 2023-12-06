use crate::model::overview::Overview;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

#[derive(Debug, Clone, Serialize)]
pub struct MonitorVec {
    pub len: usize,
    pub load_1: VecDeque<f32>,
    pub load_5: VecDeque<f32>,
    pub load_15: VecDeque<f32>,
    pub cpu_usage: VecDeque<f32>,
    pub memory_used: VecDeque<u64>,
    pub memory_free: VecDeque<u64>,
    pub swap_used: VecDeque<u64>,
    pub swap_free: VecDeque<u64>,
    pub disk_used: VecDeque<u64>,
    pub disk_read: VecDeque<u64>,
    pub disk_write: VecDeque<u64>,
    pub network_rx: VecDeque<u64>,
    pub network_tx: VecDeque<u64>,
}

impl MonitoringData {
    pub fn new(sys_data: Overview) -> Self {
        Self {
            timestamp: Utc::now().timestamp(),
            load_1: sys_data.load_avg[0] as f32,
            load_5: sys_data.load_avg[1] as f32,
            load_15: sys_data.load_avg[2] as f32,
            cpu_usage: sys_data.cpu_usage,
            memory_used: sys_data.memory_usage.used,
            memory_free: sys_data.memory_usage.free,
            swap_used: sys_data.memory_usage.swap_used,
            swap_free: sys_data.memory_usage.swap_free,
            disk_used: sys_data.disk_usage.used,
            disk_read: sys_data.disk_io.read,
            disk_write: sys_data.disk_io.write,
            network_rx: sys_data.network_io.received,
            network_tx: sys_data.network_io.transmitted,
        }
    }
}
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MonitoringData {
    pub timestamp: i64,
    pub load_1: f32,
    pub load_5: f32,
    pub load_15: f32,
    pub cpu_usage: f32,
    pub memory_used: u64,
    pub memory_free: u64,
    pub swap_used: u64,
    pub swap_free: u64,
    pub disk_used: u64,
    pub disk_read: u64,
    pub disk_write: u64,
    pub network_rx: u64,
    pub network_tx: u64,
}

impl MonitorVec {
    pub fn new(queue_len: usize) -> Self {
        let default_data_f32 = VecDeque::new();
        let default_data_u64 = VecDeque::new();
        Self {
            len: queue_len,
            load_1: default_data_f32.clone(),
            load_5: default_data_f32.clone(),
            load_15: default_data_f32.clone(),
            cpu_usage: default_data_f32.clone(),
            memory_used: default_data_u64.clone(),
            memory_free: default_data_u64.clone(),
            swap_used: default_data_u64.clone(),
            swap_free: default_data_u64.clone(),
            disk_used: default_data_u64.clone(),
            disk_read: default_data_u64.clone(),
            disk_write: default_data_u64.clone(),
            network_rx: default_data_u64.clone(),
            network_tx: default_data_u64.clone(),
        }
    }
    fn pop_front(&mut self) {
        self.load_1.pop_front();
        self.load_5.pop_front();
        self.load_15.pop_front();
        self.cpu_usage.pop_front();
        self.memory_used.pop_front();
        self.memory_free.pop_front();
        self.swap_used.pop_front();
        self.swap_free.pop_front();
        self.disk_used.pop_front();
        self.disk_read.pop_front();
        self.disk_write.pop_front();
        self.network_rx.pop_front();
        self.network_tx.pop_front();
    }
    fn push_back(&mut self, os_data: MonitoringData) {
        self.load_1.push_back(os_data.load_1.clone());
        self.load_5.push_back(os_data.load_5.clone());
        self.load_15.push_back(os_data.load_15.clone());
        self.cpu_usage.push_back(os_data.cpu_usage.clone());
        self.memory_used.push_back(os_data.memory_used.clone());
        self.memory_free.push_back(os_data.memory_free.clone());
        self.swap_used.push_back(os_data.swap_used.clone());
        self.swap_free.push_back(os_data.swap_free.clone());
        self.disk_used.push_back(os_data.disk_used.clone());
        self.disk_read.push_back(os_data.disk_read.clone());
        self.disk_write.push_back(os_data.disk_write.clone());
        self.network_rx.push_back(os_data.network_rx.clone());
        self.network_tx.push_back(os_data.network_tx.clone());
    }
    pub fn insert(&mut self, os_data: MonitoringData) {
        if self.load_1.len() >= self.len {
            self.pop_front();
        }
        self.push_back(os_data);
    }
    pub fn get_average(&self, timestamp: i64) -> MonitoringData {
        MonitoringData {
            timestamp: timestamp,
            load_1: MonitorVec::average_f32(self.load_1.clone()),
            load_5: MonitorVec::average_f32(self.load_5.clone()),
            load_15: MonitorVec::average_f32(self.load_15.clone()),
            cpu_usage: MonitorVec::average_f32(self.cpu_usage.clone()),
            memory_used: MonitorVec::average_u64(self.memory_used.clone()),
            memory_free: MonitorVec::average_u64(self.memory_free.clone()),
            swap_used: MonitorVec::average_u64(self.swap_used.clone()),
            swap_free: MonitorVec::average_u64(self.swap_free.clone()),
            disk_used: MonitorVec::average_u64(self.disk_used.clone()),
            disk_read: MonitorVec::average_u64(self.disk_read.clone()),
            disk_write: MonitorVec::average_u64(self.disk_write.clone()),
            network_rx: MonitorVec::average_u64(self.network_rx.clone()),
            network_tx: MonitorVec::average_u64(self.network_tx.clone()),
        }
    }

    fn average_f32(data: VecDeque<f32>) -> f32 {
        data.iter().cloned().sum::<f32>() / data.len() as f32
    }
    fn average_u64(data: VecDeque<u64>) -> u64 {
        data.iter().cloned().sum::<u64>() / data.len() as u64
    }
}
