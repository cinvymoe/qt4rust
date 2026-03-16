// 报警记录视图状态

use std::time::SystemTime;

/// 报警类型
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AlarmType {
    /// 预警（90-100%）
    Warning,
    
    /// 危险（>100%）
    Danger,
}

impl AlarmType {
    /// 获取报警类型的显示文本
    pub fn display_text(&self) -> &str {
        match self {
            AlarmType::Warning => "预警",
            AlarmType::Danger => "危险",
        }
    }
    
    /// 获取报警级别（数值）
    pub fn level(&self) -> u8 {
        match self {
            AlarmType::Warning => 1,
            AlarmType::Danger => 2,
        }
    }
}

/// 报警记录
#[derive(Debug, Clone, PartialEq)]
pub struct AlarmRecord {
    /// 报警 ID
    pub id: u64,
    
    /// 报警时间
    pub timestamp: SystemTime,
    
    /// 报警类型
    pub alarm_type: AlarmType,
    
    /// 力矩百分比
    pub moment_percentage: f64,
    
    /// 当前载荷（吨）
    pub current_load: f64,
    
    /// 工作半径（米）
    pub working_radius: f64,
    
    /// 吊臂角度（度）
    pub boom_angle: f64,
    
    /// 是否已读
    pub is_read: bool,
}

impl AlarmRecord {
    /// 创建新的报警记录
    pub fn new(
        id: u64,
        alarm_type: AlarmType,
        moment_percentage: f64,
        current_load: f64,
        working_radius: f64,
        boom_angle: f64,
    ) -> Self {
        Self {
            id,
            timestamp: SystemTime::now(),
            alarm_type,
            moment_percentage,
            current_load,
            working_radius,
            boom_angle,
            is_read: false,
        }
    }
    
    /// 标记为已读
    pub fn mark_as_read(&mut self) {
        self.is_read = true;
    }
}

/// 报警记录视图状态
#[derive(Debug, Clone, PartialEq)]
pub struct AlarmState {
    /// 报警记录列表
    pub records: Vec<AlarmRecord>,
    
    /// 当前选中的记录 ID
    pub selected_record_id: Option<u64>,
    
    /// 过滤器：报警类型
    pub filter_type: Option<AlarmType>,
    
    /// 是否只显示未读
    pub show_unread_only: bool,
    
    /// 是否正在加载数据
    pub is_loading: bool,
    
    /// 错误信息
    pub error_message: Option<String>,
    
    /// 总记录数（包括未加载的）
    pub total_count: usize,
}

impl Default for AlarmState {
    fn default() -> Self {
        Self {
            records: Vec::new(),
            selected_record_id: None,
            filter_type: None,
            show_unread_only: false,
            is_loading: false,
            error_message: None,
            total_count: 0,
        }
    }
}

impl AlarmState {
    /// 创建新的报警状态
    pub fn new() -> Self {
        Self::default()
    }
    
    /// 添加报警记录
    pub fn add_record(&mut self, record: AlarmRecord) {
        self.records.push(record);
        self.total_count = self.records.len();
    }
    
    /// 获取未读记录数量
    pub fn unread_count(&self) -> usize {
        self.records.iter().filter(|r| !r.is_read).count()
    }
    
    /// 获取危险级别记录数量
    pub fn danger_count(&self) -> usize {
        self.records
            .iter()
            .filter(|r| r.alarm_type == AlarmType::Danger)
            .count()
    }
    
    /// 获取预警级别记录数量
    pub fn warning_count(&self) -> usize {
        self.records
            .iter()
            .filter(|r| r.alarm_type == AlarmType::Warning)
            .count()
    }
    
    /// 标记所有记录为已读
    pub fn mark_all_as_read(&mut self) {
        for record in &mut self.records {
            record.is_read = true;
        }
    }
    
    /// 根据 ID 查找记录
    pub fn find_record(&self, id: u64) -> Option<&AlarmRecord> {
        self.records.iter().find(|r| r.id == id)
    }
    
    /// 根据 ID 查找记录（可变引用）
    pub fn find_record_mut(&mut self, id: u64) -> Option<&mut AlarmRecord> {
        self.records.iter_mut().find(|r| r.id == id)
    }
    
    /// 获取过滤后的记录
    pub fn filtered_records(&self) -> Vec<&AlarmRecord> {
        self.records
            .iter()
            .filter(|r| {
                // 类型过滤
                if let Some(ref filter_type) = self.filter_type {
                    if &r.alarm_type != filter_type {
                        return false;
                    }
                }
                
                // 未读过滤
                if self.show_unread_only && r.is_read {
                    return false;
                }
                
                true
            })
            .collect()
    }
    
    /// 清空所有记录
    pub fn clear_records(&mut self) {
        self.records.clear();
        self.total_count = 0;
        self.selected_record_id = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_alarm_type() {
        assert_eq!(AlarmType::Warning.display_text(), "预警");
        assert_eq!(AlarmType::Danger.display_text(), "危险");
        assert_eq!(AlarmType::Warning.level(), 1);
        assert_eq!(AlarmType::Danger.level(), 2);
    }
    
    #[test]
    fn test_alarm_record() {
        let mut record = AlarmRecord::new(1, AlarmType::Warning, 95.0, 20.0, 10.0, 60.0);
        assert!(!record.is_read);
        
        record.mark_as_read();
        assert!(record.is_read);
    }
    
    #[test]
    fn test_alarm_state() {
        let mut state = AlarmState::default();
        
        let record1 = AlarmRecord::new(1, AlarmType::Warning, 95.0, 20.0, 10.0, 60.0);
        let record2 = AlarmRecord::new(2, AlarmType::Danger, 105.0, 25.0, 12.0, 65.0);
        
        state.add_record(record1);
        state.add_record(record2);
        
        assert_eq!(state.records.len(), 2);
        assert_eq!(state.total_count, 2);
        assert_eq!(state.unread_count(), 2);
        assert_eq!(state.danger_count(), 1);
        assert_eq!(state.warning_count(), 1);
    }
    
    #[test]
    fn test_mark_all_as_read() {
        let mut state = AlarmState::default();
        
        state.add_record(AlarmRecord::new(1, AlarmType::Warning, 95.0, 20.0, 10.0, 60.0));
        state.add_record(AlarmRecord::new(2, AlarmType::Danger, 105.0, 25.0, 12.0, 65.0));
        
        assert_eq!(state.unread_count(), 2);
        
        state.mark_all_as_read();
        assert_eq!(state.unread_count(), 0);
    }
    
    #[test]
    fn test_filtered_records() {
        let mut state = AlarmState::default();
        
        state.add_record(AlarmRecord::new(1, AlarmType::Warning, 95.0, 20.0, 10.0, 60.0));
        state.add_record(AlarmRecord::new(2, AlarmType::Danger, 105.0, 25.0, 12.0, 65.0));
        state.add_record(AlarmRecord::new(3, AlarmType::Warning, 92.0, 18.0, 9.0, 58.0));
        
        // 过滤危险类型
        state.filter_type = Some(AlarmType::Danger);
        let filtered = state.filtered_records();
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].id, 2);
        
        // 过滤预警类型
        state.filter_type = Some(AlarmType::Warning);
        let filtered = state.filtered_records();
        assert_eq!(filtered.len(), 2);
    }
    
    #[test]
    fn test_find_record() {
        let mut state = AlarmState::default();
        
        state.add_record(AlarmRecord::new(1, AlarmType::Warning, 95.0, 20.0, 10.0, 60.0));
        state.add_record(AlarmRecord::new(2, AlarmType::Danger, 105.0, 25.0, 12.0, 65.0));
        
        let record = state.find_record(1);
        assert!(record.is_some());
        assert_eq!(record.unwrap().id, 1);
        
        let record = state.find_record(999);
        assert!(record.is_none());
    }
}
