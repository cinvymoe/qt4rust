// 通用状态定义

/// 加载状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LoadingState {
    /// 空闲状态
    Idle,
    
    /// 加载中
    Loading,
    
    /// 加载成功
    Success,
    
    /// 加载失败
    Failed,
}

impl Default for LoadingState {
    fn default() -> Self {
        Self::Idle
    }
}

impl LoadingState {
    /// 是否正在加载
    pub fn is_loading(&self) -> bool {
        matches!(self, Self::Loading)
    }
    
    /// 是否加载成功
    pub fn is_success(&self) -> bool {
        matches!(self, Self::Success)
    }
    
    /// 是否加载失败
    pub fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }
}

/// 错误状态
#[derive(Debug, Clone, PartialEq)]
pub struct ErrorState {
    /// 错误消息
    pub message: String,
    
    /// 错误代码
    pub code: Option<i32>,
    
    /// 是否可重试
    pub is_retryable: bool,
    
    /// 错误详情
    pub details: Option<String>,
}

impl ErrorState {
    /// 创建新的错误状态
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
            code: None,
            is_retryable: false,
            details: None,
        }
    }
    
    /// 创建带错误代码的错误状态
    pub fn with_code(message: impl Into<String>, code: i32) -> Self {
        Self {
            message: message.into(),
            code: Some(code),
            is_retryable: false,
            details: None,
        }
    }
    
    /// 设置为可重试
    pub fn retryable(mut self) -> Self {
        self.is_retryable = true;
        self
    }
    
    /// 添加详细信息
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

/// 分页状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaginationState {
    /// 当前页码（从 1 开始）
    pub current_page: usize,
    
    /// 每页数量
    pub page_size: usize,
    
    /// 总记录数
    pub total_count: usize,
}

impl Default for PaginationState {
    fn default() -> Self {
        Self {
            current_page: 1,
            page_size: 20,
            total_count: 0,
        }
    }
}

impl PaginationState {
    /// 创建新的分页状态
    pub fn new(page_size: usize) -> Self {
        Self {
            current_page: 1,
            page_size,
            total_count: 0,
        }
    }
    
    /// 获取总页数
    pub fn total_pages(&self) -> usize {
        if self.total_count == 0 {
            return 0;
        }
        (self.total_count + self.page_size - 1) / self.page_size
    }
    
    /// 是否有下一页
    pub fn has_next_page(&self) -> bool {
        self.current_page < self.total_pages()
    }
    
    /// 是否有上一页
    pub fn has_previous_page(&self) -> bool {
        self.current_page > 1
    }
    
    /// 获取起始索引
    pub fn start_index(&self) -> usize {
        (self.current_page - 1) * self.page_size
    }
    
    /// 获取结束索引
    pub fn end_index(&self) -> usize {
        (self.current_page * self.page_size).min(self.total_count)
    }
    
    /// 跳转到下一页
    pub fn next_page(&mut self) -> bool {
        if self.has_next_page() {
            self.current_page += 1;
            true
        } else {
            false
        }
    }
    
    /// 跳转到上一页
    pub fn previous_page(&mut self) -> bool {
        if self.has_previous_page() {
            self.current_page -= 1;
            true
        } else {
            false
        }
    }
    
    /// 跳转到指定页
    pub fn goto_page(&mut self, page: usize) -> bool {
        if page >= 1 && page <= self.total_pages() {
            self.current_page = page;
            true
        } else {
            false
        }
    }
}

/// 网络连接状态
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ConnectionState {
    /// 已连接
    Connected,
    
    /// 连接中
    Connecting,
    
    /// 已断开
    Disconnected,
    
    /// 连接错误
    Error,
}

impl Default for ConnectionState {
    fn default() -> Self {
        Self::Disconnected
    }
}

impl ConnectionState {
    /// 是否已连接
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
    }
    
    /// 是否正在连接
    pub fn is_connecting(&self) -> bool {
        matches!(self, Self::Connecting)
    }
    
    /// 是否已断开
    pub fn is_disconnected(&self) -> bool {
        matches!(self, Self::Disconnected)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_loading_state() {
        let state = LoadingState::default();
        assert_eq!(state, LoadingState::Idle);
        
        let loading = LoadingState::Loading;
        assert!(loading.is_loading());
        assert!(!loading.is_success());
        assert!(!loading.is_failed());
    }
    
    #[test]
    fn test_error_state() {
        let error = ErrorState::new("测试错误");
        assert_eq!(error.message, "测试错误");
        assert!(!error.is_retryable);
        
        let error = ErrorState::with_code("网络错误", 500)
            .retryable()
            .with_details("连接超时");
        
        assert_eq!(error.code, Some(500));
        assert!(error.is_retryable);
        assert_eq!(error.details, Some("连接超时".to_string()));
    }
    
    #[test]
    fn test_pagination_state() {
        let mut state = PaginationState::new(10);
        state.total_count = 45;
        
        assert_eq!(state.total_pages(), 5);
        assert_eq!(state.current_page, 1);
        assert!(state.has_next_page());
        assert!(!state.has_previous_page());
        
        assert_eq!(state.start_index(), 0);
        assert_eq!(state.end_index(), 10);
        
        state.next_page();
        assert_eq!(state.current_page, 2);
        assert_eq!(state.start_index(), 10);
        assert_eq!(state.end_index(), 20);
        
        state.goto_page(5);
        assert_eq!(state.current_page, 5);
        assert!(!state.has_next_page());
        assert!(state.has_previous_page());
    }
    
    #[test]
    fn test_connection_state() {
        let state = ConnectionState::default();
        assert_eq!(state, ConnectionState::Disconnected);
        
        let connected = ConnectionState::Connected;
        assert!(connected.is_connected());
        assert!(!connected.is_connecting());
        assert!(!connected.is_disconnected());
    }
}
