//! 时间感知系统 (KAIROS)
//! 
//! 这个模块实现了基于时间的智能响应系统，根据时间、日期和用户模式提供
//! 个性化的建议和响应。

use crate::state::AppState;
use chrono::{DateTime, Datelike, Duration, Timelike, Utc, Weekday};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// 时间段类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimePeriod {
    /// 凌晨 (0:00-3:00)
    Midnight,
    
    /// 深夜 (3:00-5:00)
    LateNight,
    
    /// 清晨 (5:00-7:00)
    EarlyMorning,
    
    /// 早晨 (7:00-9:00)
    Morning,
    
    /// 上午 (9:00-12:00)
    LateMorning,
    
    /// 中午 (12:00-14:00)
    Noon,
    
    /// 下午 (14:00-16:00)
    Afternoon,
    
    /// 傍晚 (16:00-18:00)
    LateAfternoon,
    
    /// 晚上 (18:00-20:00)
    Evening,
    
    /// 夜晚 (20:00-22:00)
    Night,
    
    /// 午夜 (22:00-24:00)
    LateEvening,
}

/// 用户活动模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserActivityPattern {
    /// 工作日活动模式
    pub weekday_patterns: HashMap<Weekday, Vec<TimePeriod>>,
    
    /// 最活跃的时间段
    pub most_active_periods: Vec<TimePeriod>,
    
    /// 平均会话时长（分钟）
    pub average_session_duration: f64,
    
    /// 常用工具统计
    pub tool_usage: HashMap<String, u64>,
    
    /// 记录开始时间
    pub pattern_start_date: DateTime<Utc>,
}

impl Default for UserActivityPattern {
    fn default() -> Self {
        Self {
            weekday_patterns: HashMap::new(),
            most_active_periods: Vec::new(),
            average_session_duration: 0.0,
            tool_usage: HashMap::new(),
            pattern_start_date: Utc::now(),
        }
    }
}

/// 时间感知建议
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeAwareSuggestion {
    /// 建议类型
    pub suggestion_type: TimeSuggestionType,
    
    /// 建议标题
    pub title: String,
    
    /// 详细描述
    pub description: String,
    
    /// 建议的优先级
    pub priority: u8,
    
    /// 适用的时间段
    pub applicable_periods: Vec<TimePeriod>,
    
    /// 适用的星期
    pub applicable_weekdays: Option<HashSet<Weekday>>,
}

/// 时间建议类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TimeSuggestionType {
    /// 休息提醒
    BreakReminder,
    
    /// 任务建议
    TaskSuggestion,
    
    /// 工具推荐
    ToolRecommendation,
    
    /// 工作模式切换
    WorkModeSwitch,
    
    /// 会话总结
    SessionSummary,
    
    /// 目标提醒
    GoalReminder,
}

/// Kairos 管理器
#[derive(Debug)]
pub struct KairosManager {
    /// 应用状态
    state: AppState,
    
    /// 用户活动模式
    activity_pattern: UserActivityPattern,
    
    /// 当前时间
    current_time: DateTime<Utc>,
    
    /// 会话开始时间
    session_start_time: DateTime<Utc>,
    
    /// 时间感知建议
    suggestions: Vec<TimeAwareSuggestion>,
    
    /// 是否启用自动提醒
    auto_reminders: bool,
    
    /// 提醒间隔
    reminder_interval: Duration,
}

impl KairosManager {
    /// 创建新的 Kairos 管理器
    pub fn new(state: AppState) -> Self {
        Self {
            state,
            activity_pattern: UserActivityPattern::default(),
            current_time: Utc::now(),
            session_start_time: Utc::now(),
            suggestions: Vec::new(),
            auto_reminders: true,
            reminder_interval: Duration::minutes(60),
        }
    }
    
    /// 获取当前时间段
    pub fn current_period(&self) -> TimePeriod {
        let hour = self.current_time.hour();
        match hour {
            0..=2 => TimePeriod::Midnight,
            3..=4 => TimePeriod::LateNight,
            5..=6 => TimePeriod::EarlyMorning,
            7..=8 => TimePeriod::Morning,
            9..=11 => TimePeriod::LateMorning,
            12..=13 => TimePeriod::Noon,
            14..=15 => TimePeriod::Afternoon,
            16..=17 => TimePeriod::LateAfternoon,
            18..=19 => TimePeriod::Evening,
            20..=21 => TimePeriod::Night,
            22..=23 => TimePeriod::LateEvening,
            _ => TimePeriod::Midnight,
        }
    }
    
    /// 获取当前星期几
    pub fn current_weekday(&self) -> Weekday {
        self.current_time.weekday()
    }
    
    /// 是否是工作日
    pub fn is_weekday(&self) -> bool {
        !matches!(
            self.current_weekday(),
            Weekday::Sat | Weekday::Sun
        )
    }
    
    /// 获取会话持续时间
    pub fn session_duration(&self) -> Duration {
        self.current_time.signed_duration_since(self.session_start_time)
    }
    
    /// 记录工具使用
    pub fn record_tool_usage(&mut self, tool_name: &str) {
        *self.activity_pattern.tool_usage.entry(tool_name.to_string()).or_insert(0) += 1;
    }
    
    /// 记录当前活动
    pub fn record_activity(&mut self) {
        let weekday = self.current_weekday();
        let period = self.current_period();
        
        self.activity_pattern
            .weekday_patterns
            .entry(weekday)
            .or_insert_with(Vec::new)
            .push(period);
    }
    
    /// 添加时间感知建议
    pub fn add_suggestion(&mut self, suggestion: TimeAwareSuggestion) {
        self.suggestions.push(suggestion);
    }
    
    /// 获取适用于当前时间的建议
    pub fn applicable_suggestions(&self) -> Vec<&TimeAwareSuggestion> {
        let current_period = self.current_period();
        let current_weekday = self.current_weekday();
        
        self.suggestions
            .iter()
            .filter(|s| {
                s.applicable_periods.contains(&current_period)
                    && s.applicable_weekdays
                        .as_ref()
                        .map(|w| w.contains(&current_weekday))
                        .unwrap_or(true)
            })
            .collect()
    }
    
    /// 创建休息提醒
    pub fn create_break_reminder() -> TimeAwareSuggestion {
        TimeAwareSuggestion {
            suggestion_type: TimeSuggestionType::BreakReminder,
            title: "休息提醒".to_string(),
            description: "您已经工作了一段时间，建议休息一下。".to_string(),
            priority: 5,
            applicable_periods: vec![
                TimePeriod::LateMorning,
                TimePeriod::Afternoon,
                TimePeriod::LateAfternoon,
            ],
            applicable_weekdays: None,
        }
    }
    
    /// 创建任务建议
    pub fn create_task_suggestion(
        title: String,
        description: String,
        priority: u8,
        periods: Vec<TimePeriod>,
    ) -> TimeAwareSuggestion {
        TimeAwareSuggestion {
            suggestion_type: TimeSuggestionType::TaskSuggestion,
            title,
            description,
            priority: priority.clamp(1, 10),
            applicable_periods: periods,
            applicable_weekdays: None,
        }
    }
    
    /// 创建工具推荐
    pub fn create_tool_recommendation(tool_name: &str, description: &str) -> TimeAwareSuggestion {
        TimeAwareSuggestion {
            suggestion_type: TimeSuggestionType::ToolRecommendation,
            title: format!("工具推荐: {}", tool_name),
            description: description.to_string(),
            priority: 4,
            applicable_periods: vec![
                TimePeriod::LateMorning,
                TimePeriod::Afternoon,
                TimePeriod::LateAfternoon,
            ],
            applicable_weekdays: None,
        }
    }
    
    /// 创建工作模式切换建议
    pub fn create_work_mode_switch(mode: &str) -> TimeAwareSuggestion {
        TimeAwareSuggestion {
            suggestion_type: TimeSuggestionType::WorkModeSwitch,
            title: format!("工作模式切换: {}", mode),
            description: format!("建议切换到{}模式，提高工作效率。", mode),
            priority: 3,
            applicable_periods: vec![
                TimePeriod::Morning,
                TimePeriod::LateMorning,
                TimePeriod::Afternoon,
            ],
            applicable_weekdays: None,
        }
    }
    
    /// 创建会话总结建议
    pub fn create_session_summary() -> TimeAwareSuggestion {
        TimeAwareSuggestion {
            suggestion_type: TimeSuggestionType::SessionSummary,
            title: "会话总结".to_string(),
            description: "建议总结本次会话的内容和成果。".to_string(),
            priority: 6,
            applicable_periods: vec![
                TimePeriod::Evening,
                TimePeriod::Night,
                TimePeriod::LateEvening,
            ],
            applicable_weekdays: None,
        }
    }
    
    /// 创建目标提醒
    pub fn create_goal_reminder(goal: &str) -> TimeAwareSuggestion {
        TimeAwareSuggestion {
            suggestion_type: TimeSuggestionType::GoalReminder,
            title: "目标提醒".to_string(),
            description: format!("不要忘记您的目标: {}", goal),
            priority: 7,
            applicable_periods: vec![
                TimePeriod::EarlyMorning,
                TimePeriod::Morning,
                TimePeriod::LateEvening,
            ],
            applicable_weekdays: None,
        }
    }
    
    /// 根据时间段生成智能建议
    pub fn generate_time_based_suggestions(&self) -> Vec<TimeAwareSuggestion> {
        let current_period = self.current_period();
        let mut suggestions = Vec::new();
        
        match current_period {
            TimePeriod::Midnight | TimePeriod::LateNight => {
                // 深夜建议
                suggestions.push(TimeAwareSuggestion {
                    suggestion_type: TimeSuggestionType::BreakReminder,
                    title: "深夜提醒".to_string(),
                    description: "夜深了，建议休息，保持良好的作息习惯。".to_string(),
                    priority: 8,
                    applicable_periods: vec![TimePeriod::Midnight, TimePeriod::LateNight],
                    applicable_weekdays: None,
                });
            }
            TimePeriod::EarlyMorning => {
                // 清晨建议
                suggestions.push(TimeAwareSuggestion {
                    suggestion_type: TimeSuggestionType::TaskSuggestion,
                    title: "清晨计划".to_string(),
                    description: "新的一天开始了，建议制定今日计划。".to_string(),
                    priority: 6,
                    applicable_periods: vec![TimePeriod::EarlyMorning],
                    applicable_weekdays: None,
                });
            }
            TimePeriod::Morning => {
                // 早晨建议
                suggestions.push(TimeAwareSuggestion {
                    suggestion_type: TimeSuggestionType::WorkModeSwitch,
                    title: "进入工作状态".to_string(),
                    description: "建议开始专注工作，充分利用上午的高效时间。".to_string(),
                    priority: 5,
                    applicable_periods: vec![TimePeriod::Morning],
                    applicable_weekdays: None,
                });
            }
            TimePeriod::LateMorning => {
                // 上午建议
                suggestions.push(TimeAwareSuggestion {
                    suggestion_type: TimeSuggestionType::BreakReminder,
                    title: "上午休息".to_string(),
                    description: "工作了一上午，建议短暂休息一下。".to_string(),
                    priority: 4,
                    applicable_periods: vec![TimePeriod::LateMorning],
                    applicable_weekdays: None,
                });
            }
            TimePeriod::Noon => {
                // 中午建议
                suggestions.push(TimeAwareSuggestion {
                    suggestion_type: TimeSuggestionType::WorkModeSwitch,
                    title: "午休时间".to_string(),
                    description: "午餐时间，建议适当休息，为下午的工作充电。".to_string(),
                    priority: 5,
                    applicable_periods: vec![TimePeriod::Noon],
                    applicable_weekdays: None,
                });
            }
            TimePeriod::Afternoon => {
                // 下午建议
                suggestions.push(TimeAwareSuggestion {
                    suggestion_type: TimeSuggestionType::TaskSuggestion,
                    title: "下午工作".to_string(),
                    description: "下午精力充沛，建议处理重要任务。".to_string(),
                    priority: 4,
                    applicable_periods: vec![TimePeriod::Afternoon],
                    applicable_weekdays: None,
                });
            }
            TimePeriod::LateAfternoon => {
                // 傍晚建议
                suggestions.push(TimeAwareSuggestion {
                    suggestion_type: TimeSuggestionType::BreakReminder,
                    title: "傍晚休息".to_string(),
                    description: "工作了一天，建议适当休息，调整状态。".to_string(),
                    priority: 3,
                    applicable_periods: vec![TimePeriod::LateAfternoon],
                    applicable_weekdays: None,
                });
            }
            TimePeriod::Evening => {
                // 晚上建议
                suggestions.push(TimeAwareSuggestion {
                    suggestion_type: TimeSuggestionType::SessionSummary,
                    title: "今日总结".to_string(),
                    description: "一天的工作即将结束，建议总结今日成果。".to_string(),
                    priority: 6,
                    applicable_periods: vec![TimePeriod::Evening],
                    applicable_weekdays: None,
                });
            }
            TimePeriod::Night => {
                // 夜晚建议
                suggestions.push(TimeAwareSuggestion {
                    suggestion_type: TimeSuggestionType::BreakReminder,
                    title: "夜晚提醒".to_string(),
                    description: "夜晚时光，建议放松身心，准备休息。".to_string(),
                    priority: 4,
                    applicable_periods: vec![TimePeriod::Night],
                    applicable_weekdays: None,
                });
            }
            TimePeriod::LateEvening => {
                // 午夜建议
                suggestions.push(TimeAwareSuggestion {
                    suggestion_type: TimeSuggestionType::BreakReminder,
                    title: "午夜提醒".to_string(),
                    description: "时间不早了，建议结束工作，早点休息。".to_string(),
                    priority: 7,
                    applicable_periods: vec![TimePeriod::LateEvening],
                    applicable_weekdays: None,
                });
            }
        }
        
        suggestions
    }
    
    /// 根据用户活动模式生成个性化建议
    pub fn generate_personalized_suggestions(&self) -> Vec<TimeAwareSuggestion> {
        let mut suggestions = Vec::new();
        
        // 分析用户最活跃的时间段
        if !self.activity_pattern.most_active_periods.is_empty() {
            let most_active = &self.activity_pattern.most_active_periods[0];
            suggestions.push(TimeAwareSuggestion {
                suggestion_type: TimeSuggestionType::WorkModeSwitch,
                title: "高效时间提醒".to_string(),
                description: format!("这是您的高效工作时间，建议处理重要任务。"),
                priority: 5,
                applicable_periods: vec![*most_active],
                applicable_weekdays: None,
            });
        }
        
        // 分析工具使用情况
        if let Some((most_used_tool, _)) = self.activity_pattern.tool_usage.iter().max_by_key(|&(_, count)| count) {
            suggestions.push(TimeAwareSuggestion {
                suggestion_type: TimeSuggestionType::ToolRecommendation,
                title: format!("常用工具提醒"),
                description: format!("您经常使用 {}，这是一个很好的工具！", most_used_tool),
                priority: 3,
                applicable_periods: vec![
                    TimePeriod::LateMorning,
                    TimePeriod::Afternoon,
                    TimePeriod::LateAfternoon,
                ],
                applicable_weekdays: None,
            });
        }
        
        // 分析会话时长
        if self.activity_pattern.average_session_duration > 60.0 {
            suggestions.push(TimeAwareSuggestion {
                suggestion_type: TimeSuggestionType::BreakReminder,
                title: "长时间会话提醒".to_string(),
                description: "您的平均会话时长较长，建议适当休息。".to_string(),
                priority: 4,
                applicable_periods: vec![
                    TimePeriod::LateMorning,
                    TimePeriod::LateAfternoon,
                ],
                applicable_weekdays: None,
            });
        }
        
        suggestions
    }
    
    /// 生成所有智能建议
    pub fn generate_suggestions(&mut self) -> Vec<TimeAwareSuggestion> {
        let mut suggestions = self.generate_time_based_suggestions();
        let personalized = self.generate_personalized_suggestions();
        suggestions.extend(personalized);
        
        // 更新建议列表
        self.suggestions = suggestions.clone();
        suggestions
    }
    
    /// 获取优先级最高的建议
    pub fn get_highest_priority_suggestion(&self) -> Option<&TimeAwareSuggestion> {
        self.applicable_suggestions()
            .into_iter()
            .max_by_key(|s| s.priority)
    }
    
    /// 分析用户活动模式
    pub fn analyze_activity_pattern(&mut self) {
        // 分析工作日活动模式
        for (weekday, periods) in &self.activity_pattern.weekday_patterns {
            // 统计每个时间段的出现次数
            let mut period_counts = HashMap::new();
            for period in periods {
                *period_counts.entry(period).or_insert(0) += 1;
            }
            
            // 找出最活跃的时间段
            if let Some((most_active, _)) = period_counts.iter().max_by_key(|&(_, count)| count) {
                self.activity_pattern.most_active_periods.push(*most_active);
            }
        }
        
        // 计算平均会话时长
        if !self.activity_pattern.weekday_patterns.is_empty() {
            let total_sessions = self.activity_pattern.weekday_patterns.values().map(|v| v.len()).sum::<usize>();
            if total_sessions > 0 {
                // 假设每个活动记录代表15分钟
                self.activity_pattern.average_session_duration = (total_sessions * 15) as f64 / self.activity_pattern.weekday_patterns.len() as f64;
            }
        }
    }
    
    /// 更新当前时间
    pub fn update_time(&mut self) {
        self.current_time = Utc::now();
    }
    
    /// 设置自动提醒
    pub fn set_auto_reminders(&mut self, enabled: bool) {
        self.auto_reminders = enabled;
    }
    
    /// 设置提醒间隔
    pub fn set_reminder_interval(&mut self, interval: Duration) {
        self.reminder_interval = interval;
    }
    
    /// 检查是否应该显示提醒
    pub fn should_show_reminder(&self) -> bool {
        if !self.auto_reminders {
            return false;
        }
        
        let elapsed = self.session_duration();
        let current_period = self.current_period();
        
        // 根据时间段调整提醒频率
        let adjusted_interval = match current_period {
            // 深夜和凌晨，减少提醒
            TimePeriod::Midnight | TimePeriod::LateNight => self.reminder_interval + Duration::minutes(30),
            // 工作时间，保持正常提醒
            TimePeriod::EarlyMorning | TimePeriod::Morning | TimePeriod::LateMorning | 
            TimePeriod::Noon | TimePeriod::Afternoon | TimePeriod::LateAfternoon => self.reminder_interval,
            // 晚上，减少提醒
            TimePeriod::Evening | TimePeriod::Night | TimePeriod::LateEvening => self.reminder_interval + Duration::minutes(15),
        };
        
        elapsed >= adjusted_interval
    }
    
    /// 获取当前时间的智能问候语
    pub fn get_intelligent_greeting(&self) -> String {
        let greeting = get_time_greeting();
        let current_period = self.current_period();
        
        match current_period {
            TimePeriod::EarlyMorning => format!("{} 新的一天开始了，祝您工作顺利！", greeting),
            TimePeriod::Morning => format!("{} 早上好，今天有什么计划？", greeting),
            TimePeriod::LateMorning => format!("{} 上午好，工作进展如何？", greeting),
            TimePeriod::Noon => format!("{} 午餐时间到了，好好休息一下！", greeting),
            TimePeriod::Afternoon => format!("{} 下午好，继续加油！", greeting),
            TimePeriod::LateAfternoon => format!("{} 傍晚了，一天的工作即将结束。", greeting),
            TimePeriod::Evening => format!("{} 晚上好，今天过得怎么样？", greeting),
            TimePeriod::Night => format!("{} 夜深了，工作辛苦了！", greeting),
            TimePeriod::LateEvening => format!("{} 时间不早了，早点休息吧。", greeting),
            TimePeriod::Midnight | TimePeriod::LateNight => format!("{} 夜深了，注意休息！", greeting),
        }
    }
    
    /// 获取时间相关的建议
    pub fn get_time_suggestion(&mut self) -> Option<TimeAwareSuggestion> {
        // 生成建议
        self.generate_suggestions();
        
        // 获取优先级最高的建议
        self.get_highest_priority_suggestion().cloned()
    }
}

/// 获取一天中的问候语
pub fn get_time_greeting() -> &'static str {
    let hour = Utc::now().hour();
    match hour {
        0..=4 => "夜深了，注意休息！",
        5..=8 => "早上好！",
        9..=11 => "上午好！",
        12..=13 => "中午好！",
        14..=17 => "下午好！",
        18..=19 => "晚上好！",
        20..=23 => "夜深了，工作辛苦了！",
        _ => "你好！",
    }
}
