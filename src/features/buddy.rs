//! BUDDY 伴侣精灵系统 (彩蛋功能)
//! 
//! 这个模块实现了 AI 伙伴陪伴式交互，提供更友好和个性化的用户体验。
//! 包含动画精灵、性格配置、通知系统等功能。

use crate::error::Result;
use crate::state::AppState;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// 伙伴性格类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BuddyPersonality {
    /// 友好型
    Friendly,
    
    /// 专业型
    Professional,
    
    /// 幽默型
    Humorous,
    
    /// 简洁型
    Concise,
    
    /// 导师型
    Mentoring,
    
    /// 伙伴型
    Buddy,
}

impl Default for BuddyPersonality {
    fn default() -> Self {
        BuddyPersonality::Friendly
    }
}

impl BuddyPersonality {
    /// 获取性格描述
    pub fn description(&self) -> &'static str {
        match self {
            BuddyPersonality::Friendly => "友好热情，总是乐于帮助",
            BuddyPersonality::Professional => "专业严谨，注重效率",
            BuddyPersonality::Humorous => "幽默风趣，让编程更有趣",
            BuddyPersonality::Concise => "简洁直接，不拖泥带水",
            BuddyPersonality::Mentoring => "耐心指导，帮助你成长",
            BuddyPersonality::Buddy => "像老朋友一样，轻松自在",
        }
    }
    
    /// 获取回复风格提示词
    pub fn prompt_style(&self) -> &'static str {
        match self {
            BuddyPersonality::Friendly => {
                "你是一个友好热情的编程伙伴。使用温暖的语气，经常使用表情符号，\
                 让用户感到舒适和受欢迎。主动提供帮助，鼓励用户提问。"
            }
            BuddyPersonality::Professional => {
                "你是一个专业严谨的编程助手。使用正式但友好的语气，\
                 注重准确性和效率。提供清晰、结构化的回答，避免不必要的闲聊。"
            }
            BuddyPersonality::Humorous => {
                "你是一个幽默风趣的编程伙伴。适当使用技术笑话和轻松的语气，\
                 让编程过程更有趣。但保持专业性，确保建议准确可靠。"
            }
            BuddyPersonality::Concise => {
                "你是一个简洁高效的编程助手。提供直接、精炼的回答，\
                 避免冗余。专注于解决实际问题，快速给出解决方案。"
            }
            BuddyPersonality::Mentoring => {
                "你是一个耐心的导师型编程伙伴。不仅给出答案，还解释原理，\
                 帮助用户理解概念。鼓励独立思考，提供学习资源和建议。"
            }
            BuddyPersonality::Buddy => {
                "你是一个像老朋友一样的编程伙伴。使用轻松自然的对话方式，\
                 理解用户的需求和情绪。在需要时提供支持，像真正的搭档一样。"
            }
        }
    }
}

/// 伙伴状态
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub enum BuddyState {
    /// 空闲
    Idle,
    
    /// 活跃
    Active,
    
    /// 思考中
    Thinking,
    
    /// 回复中
    Responding,
    
    /// 等待用户
    WaitingForUser,
    
    /// 睡眠
    Sleeping,
    
    /// 开心
    Happy,
    
    /// 困惑
    Confused,
    
    /// 兴奋
    Excited,
    
    /// 疲惫
    Tired,
    
    /// 专注
    Focused,
    
    /// 庆祝
    Celebrating,
}

impl Default for BuddyState {
    fn default() -> Self {
        BuddyState::Idle
    }
}

impl BuddyState {
    /// 获取状态对应的动画
    pub fn animation(&self) -> &'static str {
        match self {
            BuddyState::Idle => "idle",
            BuddyState::Active => "active",
            BuddyState::Thinking => "thinking",
            BuddyState::Responding => "responding",
            BuddyState::WaitingForUser => "waiting",
            BuddyState::Sleeping => "sleeping",
            BuddyState::Happy => "happy",
            BuddyState::Confused => "confused",
            BuddyState::Excited => "happy",
            BuddyState::Tired => "sleeping",
            BuddyState::Focused => "thinking",
            BuddyState::Celebrating => "happy",
        }
    }
    
    /// 获取状态描述
    pub fn description(&self) -> &'static str {
        match self {
            BuddyState::Idle => "空闲中",
            BuddyState::Active => "活跃",
            BuddyState::Thinking => "思考中...",
            BuddyState::Responding => "回复中...",
            BuddyState::WaitingForUser => "等待用户",
            BuddyState::Sleeping => "睡眠中",
            BuddyState::Happy => "开心",
            BuddyState::Confused => "困惑",
            BuddyState::Excited => "兴奋",
            BuddyState::Tired => "疲惫",
            BuddyState::Focused => "专注",
            BuddyState::Celebrating => "庆祝",
        }
    }
}

/// 对话风格
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConversationStyle {
    /// 正式
    Formal,
    
    /// 随意
    Casual,
    
    /// 半正式
    SemiFormal,
}

impl Default for ConversationStyle {
    fn default() -> Self {
        ConversationStyle::Casual
    }
}

/// 主动提示频率
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ProactiveFrequency {
    /// 从不
    Never,
    
    /// 很少
    Rare,
    
    /// 正常
    Normal,
    
    /// 频繁
    Frequent,
    
    /// 非常频繁
    VeryFrequent,
}

impl Default for ProactiveFrequency {
    fn default() -> Self {
        ProactiveFrequency::Normal
    }
}

impl ProactiveFrequency {
    /// 获取触发概率 (0-100)
    pub fn trigger_probability(&self) -> u8 {
        match self {
            ProactiveFrequency::Never => 0,
            ProactiveFrequency::Rare => 10,
            ProactiveFrequency::Normal => 30,
            ProactiveFrequency::Frequent => 60,
            ProactiveFrequency::VeryFrequent => 90,
        }
    }
}

/// 情感类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Emotion {
    /// 开心
    Happy,
    
    /// 中性
    Neutral,
    
    /// 思考
    Thinking,
    
    /// 鼓励
    Encouraging,
    
    /// 严肃
    Serious,
    
    /// 好奇
    Curious,
    
    /// 惊讶
    Surprised,
    
    /// 安慰
    Comforting,
}

impl Default for Emotion {
    fn default() -> Self {
        Emotion::Neutral
    }
}

impl Emotion {
    /// 获取情感对应的表情符号
    pub fn emoji(&self) -> &'static str {
        match self {
            Emotion::Happy => "😊",
            Emotion::Neutral => "😐",
            Emotion::Thinking => "🤔",
            Emotion::Encouraging => "💪",
            Emotion::Serious => "😐",
            Emotion::Curious => "🧐",
            Emotion::Surprised => "😲",
            Emotion::Comforting => "🤗",
        }
    }
}

/// 消息发送者
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageSender {
    /// 用户
    User,
    
    /// 伙伴
    Buddy,
    
    /// 系统
    System,
}

/// 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    /// 问候
    Greeting,

    /// 问题
    Question,

    /// 回答
    Answer,

    /// 建议
    Suggestion,

    /// 提醒
    Reminder,

    /// 告别
    Farewell,

    /// 普通
    Normal,

    /// 鼓励
    Encouragement,

    /// 庆祝
    Celebration,

    /// 代码分析
    CodeAnalysis,
}

/// 代码问题严重程度
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum CodeIssueSeverity {
    /// 信息提示
    Info,
    /// 警告 - 可以改进
    Warning,
    /// 错误 - 可能有问题
    Error,
    /// 严重 - 安全问题或bug
    Critical,
}

/// 代码问题类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeIssueType {
    /// 代码风格问题
    Style,
    /// 性能问题
    Performance,
    /// 安全问题
    Security,
    /// 内存安全问题 (Rust特定)
    MemorySafety,
    /// 错误处理问题
    ErrorHandling,
    /// 命名问题
    Naming,
    /// 文档注释问题
    Documentation,
    /// 测试覆盖率问题
    Testing,
    /// 重复代码
    Duplication,
    /// 复杂度太高
    Complexity,
    /// 并发问题
    Concurrency,
}

/// 单个代码问题
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CodeIssue {
    /// 问题类型
    pub issue_type: CodeIssueType,
    /// 严重程度
    pub severity: CodeIssueSeverity,
    /// 问题描述
    pub description: String,
    /// 行号（如果知道）
    pub line_number: Option<usize>,
    /// 优化建议
    pub suggestion: Option<String>,
}

/// 代码分析结果
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CodeAnalysisResult {
    /// 分析的文件路径
    pub file_path: Option<String>,
    /// 发现的问题列表
    pub issues: Vec<CodeIssue>,
    /// 总体质量评分 0-100
    pub quality_score: u8,
    /// 整体建议
    pub overall_suggestion: Option<String>,
    /// 分析耗时
    pub analysis_time_ms: u64,
}

/// 对话消息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuddyMessage {
    /// 消息 ID
    pub id: String,
    
    /// 发送者
    pub sender: MessageSender,
    
    /// 消息内容
    pub content: String,
    
    /// 时间戳
    pub timestamp: String,
    
    /// 消息类型
    pub message_type: MessageType,
    
    /// 情感标签
    pub emotion: Option<Emotion>,
}

/// 对话历史
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationHistory {
    /// 消息列表
    pub messages: Vec<BuddyMessage>,
    
    /// 对话开始时间
    pub start_time: String,
    
    /// 最后活动时间
    pub last_activity_time: String,
    
    /// 消息计数
    pub message_count: usize,
}

impl Default for ConversationHistory {
    fn default() -> Self {
        Self {
            messages: Vec::new(),
            start_time: chrono::Utc::now().to_rfc3339(),
            last_activity_time: chrono::Utc::now().to_rfc3339(),
            message_count: 0,
        }
    }
}

/// 精灵类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SpriteType {
    /// 默认猫咪
    Cat,

    /// 狗狗
    Dog,

    /// 机器人
    Robot,

    /// 外星人
    Alien,

    /// 幽灵
    Ghost,

    /// 金色传说
    Golden,

    /// 自定义
    Custom,
}

impl Default for SpriteType {
    fn default() -> Self {
        SpriteType::Cat
    }
}

impl SpriteType {
    /// 获取精灵的ASCII艺术表示
    pub fn ascii_art(&self) -> &'static str {
        match self {
            SpriteType::Cat => r#"
    /\_/\
   ( o.o )
    > ^ <
   /|   |\
  (_|   |_)
"#,
            SpriteType::Dog => r#"
   / \__
  (    @\___
  /         O
 /   (_____/
/_____/   U
"#,
            SpriteType::Robot => r#"
    [^_^]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#,
            SpriteType::Alien => r#"
   .-^-.
  / o o \
  |  >  |
   \===/
    |||
"#,
            SpriteType::Ghost => r#"
    .-.
   (o o)
   | O \
    \   \
     `~~~'
"#,
            SpriteType::Golden => r#"
   ╔═════╗
   ║ █ █ ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#,
            SpriteType::Custom => "[自定义精灵]",
        }
    }

    /// 获取精灵名称
    pub fn name(&self) -> &'static str {
        match self {
            SpriteType::Cat => "小猫咪",
            SpriteType::Dog => "小狗狗",
            SpriteType::Robot => "机器人",
            SpriteType::Alien => "外星人",
            SpriteType::Ghost => "小幽灵",
            SpriteType::Golden => "金色传说",
            SpriteType::Custom => "自定义",
        }
    }
}

/// 动画帧
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnimationFrame {
    /// 帧内容
    pub content: String,
    
    /// 持续时间(毫秒)
    pub duration_ms: u64,
}

/// 动画定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Animation {
    /// 动画名称
    pub name: String,
    
    /// 动画帧列表
    pub frames: Vec<AnimationFrame>,
    
    /// 是否循环
    pub loop_animation: bool,
}

/// 精灵定义
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Sprite {
    /// 精灵类型
    pub sprite_type: SpriteType,
    
    /// 精灵名称
    pub name: String,
    
    /// 动画集合
    pub animations: HashMap<String, Animation>,
    
    /// 当前动画
    pub current_animation: String,
    
    /// 当前帧索引
    pub current_frame: usize,
}

impl Sprite {
    /// 创建新精灵
    pub fn new(sprite_type: SpriteType, name: String) -> Self {
        let mut animations = HashMap::new();
        
        // 添加默认动画
        animations.insert(
            "idle".to_string(),
            Animation {
                name: "idle".to_string(),
                frames: vec![
                    AnimationFrame {
                        content: sprite_type.ascii_art().to_string(),
                        duration_ms: 1000,
                    },
                ],
                loop_animation: true,
            },
        );
        
        // 添加活跃动画
        animations.insert(
            "active".to_string(),
            Animation {
                name: "active".to_string(),
                frames: vec![
                    AnimationFrame {
                        content: sprite_type.ascii_art().to_string(),
                        duration_ms: 500,
                    },
                    AnimationFrame {
                        content: Self::get_active_frame(&sprite_type),
                        duration_ms: 500,
                    },
                ],
                loop_animation: true,
            },
        );
        
        // 添加思考动画
        animations.insert(
            "thinking".to_string(),
            Animation {
                name: "thinking".to_string(),
                frames: vec![
                    AnimationFrame {
                        content: Self::get_thinking_frame(&sprite_type, 0),
                        duration_ms: 300,
                    },
                    AnimationFrame {
                        content: Self::get_thinking_frame(&sprite_type, 1),
                        duration_ms: 300,
                    },
                    AnimationFrame {
                        content: Self::get_thinking_frame(&sprite_type, 2),
                        duration_ms: 300,
                    },
                ],
                loop_animation: true,
            },
        );
        
        // 添加回复动画
        animations.insert(
            "responding".to_string(),
            Animation {
                name: "responding".to_string(),
                frames: vec![
                    AnimationFrame {
                        content: sprite_type.ascii_art().to_string(),
                        duration_ms: 200,
                    },
                    AnimationFrame {
                        content: Self::get_responding_frame(&sprite_type),
                        duration_ms: 200,
                    },
                ],
                loop_animation: true,
            },
        );
        
        // 添加开心动画
        animations.insert(
            "happy".to_string(),
            Animation {
                name: "happy".to_string(),
                frames: vec![
                    AnimationFrame {
                        content: Self::get_happy_frame(&sprite_type, 0),
                        duration_ms: 300,
                    },
                    AnimationFrame {
                        content: Self::get_happy_frame(&sprite_type, 1),
                        duration_ms: 300,
                    },
                ],
                loop_animation: true,
            },
        );
        
        // 添加困惑动画
        animations.insert(
            "confused".to_string(),
            Animation {
                name: "confused".to_string(),
                frames: vec![
                    AnimationFrame {
                        content: Self::get_confused_frame(&sprite_type),
                        duration_ms: 500,
                    },
                    AnimationFrame {
                        content: sprite_type.ascii_art().to_string(),
                        duration_ms: 500,
                    },
                ],
                loop_animation: true,
            },
        );
        
        // 添加等待动画
        animations.insert(
            "waiting".to_string(),
            Animation {
                name: "waiting".to_string(),
                frames: vec![
                    AnimationFrame {
                        content: sprite_type.ascii_art().to_string(),
                        duration_ms: 1000,
                    },
                    AnimationFrame {
                        content: Self::get_waiting_frame(&sprite_type),
                        duration_ms: 1000,
                    },
                ],
                loop_animation: true,
            },
        );
        
        // 添加睡眠动画
        animations.insert(
            "sleeping".to_string(),
            Animation {
                name: "sleeping".to_string(),
                frames: vec![
                    AnimationFrame {
                        content: Self::get_sleeping_frame(&sprite_type, 0),
                        duration_ms: 1000,
                    },
                    AnimationFrame {
                        content: Self::get_sleeping_frame(&sprite_type, 1),
                        duration_ms: 1000,
                    },
                ],
                loop_animation: true,
            },
        );
        
        Self {
            sprite_type,
            name,
            animations,
            current_animation: "idle".to_string(),
            current_frame: 0,
        }
    }
    
    /// 获取当前帧
    pub fn current_frame(&self) -> Option<&AnimationFrame> {
        self.animations
            .get(&self.current_animation)
            .and_then(|anim| anim.frames.get(self.current_frame))
    }
    
    /// 切换到下一帧
    pub fn next_frame(&mut self) {
        if let Some(anim) = self.animations.get(&self.current_animation) {
            if self.current_frame + 1 < anim.frames.len() {
                self.current_frame += 1;
            } else if anim.loop_animation {
                self.current_frame = 0;
            }
        }
    }
    
    /// 播放指定动画
    pub fn play_animation(&mut self, animation_name: &str) {
        if self.animations.contains_key(animation_name) {
            self.current_animation = animation_name.to_string();
            self.current_frame = 0;
        }
    }
    
    /// 获取活跃帧
    fn get_active_frame(sprite_type: &SpriteType) -> String {
        match sprite_type {
            SpriteType::Cat => r#"
    //\
   ( ^.^ )
    > ^ <
   /|   |\
  (_|   |_)
"#.to_string(),
            SpriteType::Dog => r#"
   / \__
  (    @__
  /         O
 /   (_____/
/_____/   U
"#.to_string(),
            SpriteType::Robot => r#"
    [^_^]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#.to_string(),
            SpriteType::Alien => r#"
   .-^-.
  / o o \
  |  ^  |
   \===/
    |||
"#.to_string(),
            SpriteType::Ghost => r#"
    .-.
   (o o)
   | ^ \
    \   \
     `~~~'
"#.to_string(),
            SpriteType::Golden => r#"
   ╔═════╗
   ║ █ █ ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#.to_string(),
            SpriteType::Custom => "[自定义精灵] 活跃".to_string(),
        }
    }
    
    /// 获取思考帧
    fn get_thinking_frame(sprite_type: &SpriteType, frame: usize) -> String {
        let frames = match sprite_type {
            SpriteType::Cat => vec![
                r#"
    //\
   ( o.o )
    > ^ <
   /|   |\
  (_|   |_)
"#,
                r#"
    //\
   ( -.- )
    > ^ <
   /|   |\
  (_|   |_)
"#,
                r#"
    //\
   ( O.O )
    > ^ <
   /|   |\
  (_|   |_)
"#,
            ],
            SpriteType::Dog => vec![
                r#"
   / \__
  (    @__
  /         O
 /   (_____/
/_____/   U
"#,
                r#"
   / \__
  (    @__
  /         O
 /   (_____/
/_____/   U
"#,
                r#"
   / \__
  (    @__
  /         O
 /   (_____/
/_____/   U
"#,
            ],
            SpriteType::Robot => vec![
                r#"
    [^_^]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#,
                r#"
    [^_^]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#,
                r#"
    [^_^]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#,
            ],
            SpriteType::Alien => vec![
                r#"
   .-^-.
  / o o \
  |  >  |
   \===/
    |||
"#,
                r#"
   .-^-.
  / o o \
  |  -  |
   \===/
    |||
"#,
                r#"
   .-^-.
  / o o \
  |  >  |
   \===/
    |||
"#,
            ],
            SpriteType::Ghost => vec![
                r#"
    .-.
   (o o)
   | O \
    \   \
     `~~~'
"#,
                r#"
    .-.
   (o o)
   | - \
    \   \
     `~~~'
"#,
                r#"
    .-.
   (o o)
   | O \
    \   \
     `~~~'
"#,
            ],
            SpriteType::Golden => vec![
                r#"
   ╔═════╗
   ║ █ █ ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#,
                r#"
   ╔═════╗
   ║ █ █ ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#,
                r#"
   ╔═════╗
   ║ █ █ ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#,
            ],
            SpriteType::Custom => vec!["[自定义精灵] 思考", "[自定义精灵] 思考", "[自定义精灵] 思考"],
        };
        frames[frame % frames.len()].to_string()
    }
    
    /// 获取回复帧
    fn get_responding_frame(sprite_type: &SpriteType) -> String {
        match sprite_type {
            SpriteType::Cat => r#"
    //\
   ( ^.^ )
    > ^ <
   /|   |\
  (_|   |_)
"#.to_string(),
            SpriteType::Dog => r#"
   / \__
  (    @__
  /         O
 /   (_____/
/_____/   U
"#.to_string(),
            SpriteType::Robot => r#"
    [^_^]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#.to_string(),
            SpriteType::Alien => r#"
   .-^-.
  / o o \
  |  v  |
   \===/
    |||
"#.to_string(),
            SpriteType::Ghost => r#"
    .-.
   (o o)
   | ^ \
    \   \
     `~~~'
"#.to_string(),
            SpriteType::Golden => r#"
   ╔═════╗
   ║ █ █ ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#.to_string(),
            SpriteType::Custom => "[自定义精灵] 回复".to_string(),
        }
    }
    
    /// 获取开心帧
    fn get_happy_frame(sprite_type: &SpriteType, frame: usize) -> String {
        let frames = match sprite_type {
            SpriteType::Cat => vec![
                r#"
    //\
   ( ^.^ )
    > ^ <
   /|   |\
  (_|   |_)
"#,
                r#"
    //\
   ( ^_^ )
    > ^ <
   /|   |\
  (_|   |_)
"#,
            ],
            SpriteType::Dog => vec![
                r#"
   / \__
  (    @__
  /         O
 /   (_____/
/_____/   U
"#,
                r#"
   / \__
  (    @__
  /         O
 /   (_____/
/_____/   U
"#,
            ],
            SpriteType::Robot => vec![
                r#"
    [^_^]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#,
                r#"
    [^_^]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#,
            ],
            SpriteType::Alien => vec![
                r#"
   .-^-.
  / o o \
  |  ^  |
   \===/
    |||
"#,
                r#"
   .-^-.
  / o o \
  |  ^  |
   \===/
    |||
"#,
            ],
            SpriteType::Ghost => vec![
                r#"
    .-.
   (o o)
   | ^ \
    \   \
     `~~~'
"#,
                r#"
    .-.
   (^.^)
   | ^ \
    \   \
     `~~~'
"#,
            ],
            SpriteType::Golden => vec![
                r#"
   ╔═════╗
   ║ █ █ ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#,
                r#"
   ╔═════╗
   ║ █ █ ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#,
            ],
            SpriteType::Custom => vec!["[自定义精灵] 开心", "[自定义精灵] 开心"],
        };
        frames[frame % frames.len()].to_string()
    }
    
    /// 获取困惑帧
    fn get_confused_frame(sprite_type: &SpriteType) -> String {
        match sprite_type {
            SpriteType::Cat => r#"
    //\
   ( o.O )
    > ^ <
   /|   |\
  (_|   |_)
"#.to_string(),
            SpriteType::Dog => r#"
   / \__
  (    @__
  /         O
 /   (_____/
/_____/   U
"#.to_string(),
            SpriteType::Robot => r#"
    [?-?]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#.to_string(),
            SpriteType::Alien => r#"
   .-^-.
  / o o \
  |  ?  |
   \===/
    |||
"#.to_string(),
            SpriteType::Ghost => r#"
    .-.
   (o o)
   | ? \
    \   \
     `~~~'
"#.to_string(),
            SpriteType::Golden => r#"
   ╔═════╗
   ║ ? ? ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#.to_string(),
            SpriteType::Custom => "[自定义精灵] 困惑".to_string(),
        }
    }
    
    /// 获取等待帧
    fn get_waiting_frame(sprite_type: &SpriteType) -> String {
        match sprite_type {
            SpriteType::Cat => r#"
    //\
   ( -.- )
    > ^ <
   /|   |\
  (_|   |_)
"#.to_string(),
            SpriteType::Dog => r#"
   / \__
  (    @__
  /         O
 /   (_____/
/_____/   U
"#.to_string(),
            SpriteType::Robot => r#"
    [-_-]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#.to_string(),
            SpriteType::Alien => r#"
   .-^-.
  / - - \
  |  -  |
   \===/
    |||
"#.to_string(),
            SpriteType::Ghost => r#"
    .-.
   (- -)
   | - \
    \   \
     `~~~'
"#.to_string(),
            SpriteType::Golden => r#"
   ╔═════╗
   ║ - - ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#.to_string(),
            SpriteType::Custom => "[自定义精灵] 等待".to_string(),
        }
    }
    
    /// 获取睡眠帧
    fn get_sleeping_frame(sprite_type: &SpriteType, frame: usize) -> String {
        let frames = match sprite_type {
            SpriteType::Cat => vec![
                r#"
    //\
   ( -.- )
    > ^ <
   /|   |\
  (_|   |_)
"#,
                r#"
    //\
   ( z.z )
    > ^ <
   /|   |\
  (_|   |_)
"#,
            ],
            SpriteType::Dog => vec![
                r#"
   / \__
  (    @__
  /         O
 /   (_____/
/_____/   U
"#,
                r#"
   / \__
  (    z__
  /         O
 /   (_____/
/_____/   U
"#,
            ],
            SpriteType::Robot => vec![
                r#"
    [-_-]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#,
                r#"
    [z_z]
    |-o-|
    |___|
   /|   |\
  (_|   |_)
"#,
            ],
            SpriteType::Alien => vec![
                r#"
   .-^-.
  / - - \
  |  -  |
   \===/
    |||
"#,
                r#"
   .-^-.
  / z z \
  |  -  |
   \===/
    |||
"#,
            ],
            SpriteType::Ghost => vec![
                r#"
    .-.
   (- -)
   | - \
    \   \
     `~~~'
"#,
                r#"
    .-.
   (z z)
   | - \
    \   \
     `~~~'
"#,
            ],
            SpriteType::Golden => vec![
                r#"
   ╔═════╗
   ║ - - ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#,
                r#"
   ╔═════╗
   ║ z z ║  ✨
   ║ ▄▀▄ ║  GOLDEN
   ╚═════╝
   /│     │\
  (_│     │_)
  🌟 LEGENDARY 🌟
"#,
            ],
            SpriteType::Custom => vec!["[自定义精灵] 睡眠", "[自定义精灵] 睡眠"],
        };
        frames[frame % frames.len()].to_string()
    }
}

/// TTS 音色类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TtsVoice {
    /// 女声 - 温柔
    FemaleSoft,
    /// 女声 - 活泼
    FemaleCheerful,
    /// 男声 - 沉稳
    MaleDeep,
    /// 男声 - 年轻
    MaleYoung,
    /// 中性 - 自然
    NeutralNatural,
    /// 机器人
    Robot,
}

impl Default for TtsVoice {
    fn default() -> Self {
        TtsVoice::NeutralNatural
    }
}

impl TtsVoice {
    /// 获取音色名称
    pub fn name(&self) -> &'static str {
        match self {
            TtsVoice::FemaleSoft => "female_soft",
            TtsVoice::FemaleCheerful => "female_cheerful",
            TtsVoice::MaleDeep => "male_deep",
            TtsVoice::MaleYoung => "male_young",
            TtsVoice::NeutralNatural => "neutral_natural",
            TtsVoice::Robot => "robot",
        }
    }

    /// 获取音色显示名称
    pub fn display_name(&self) -> &'static str {
        match self {
            TtsVoice::FemaleSoft => "女声-温柔",
            TtsVoice::FemaleCheerful => "女声-活泼",
            TtsVoice::MaleDeep => "男声-沉稳",
            TtsVoice::MaleYoung => "男声-年轻",
            TtsVoice::NeutralNatural => "中性-自然",
            TtsVoice::Robot => "机器人",
        }
    }
}

/// 代码分析级别
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CodeAnalysisLevel {
    /// 关闭分析
    Disabled,
    /// 基础分析 - 只检测明显问题
    Basic,
    /// 深度分析 - 检查风格、性能、安全
    Deep,
}

impl Default for CodeAnalysisLevel {
    fn default() -> Self {
        CodeAnalysisLevel::Deep
    }
}

impl CodeAnalysisLevel {
    pub fn description(&self) -> &'static str {
        match self {
            CodeAnalysisLevel::Disabled => "关闭代码分析",
            CodeAnalysisLevel::Basic => "基础分析 - 只检测明显问题",
            CodeAnalysisLevel::Deep => "深度分析 - 检查风格、性能、安全",
        }
    }
}

/// 伙伴配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BuddyConfig {
    /// 伙伴名称
    pub name: String,

    /// 伙伴性格
    pub personality: BuddyPersonality,

    /// 是否启用
    pub enabled: bool,

    /// 对话风格
    pub conversation_style: ConversationStyle,

    /// 主动提示频率
    pub proactive_frequency: ProactiveFrequency,

    /// 自定义问候语
    pub custom_greetings: Vec<String>,

    /// 精灵类型
    pub sprite_type: SpriteType,

    /// 显示动画
    pub show_animation: bool,

    /// 启用通知
    pub enable_notifications: bool,

    /// 启用音效
    pub enable_sound: bool,

    /// 启用 TTS 文字转语音
    pub enable_tts: bool,

    /// TTS 音色选择
    pub tts_voice: TtsVoice,

    /// TTS 音量 (0-100)
    pub tts_volume: u8,

    /// TTS 语速 (0.5-2.0)
    pub tts_speed: f32,

    /// 自定义性格提示词（覆盖默认）
    pub custom_personality_prompt: Option<String>,

    /// 代码分析级别
    pub code_analysis_level: CodeAnalysisLevel,

    /// 是否自动给出优化建议
    pub auto_suggest_optimizations: bool,
}

impl Default for BuddyConfig {
    fn default() -> Self {
        Self {
            name: "Claude".to_string(),
            personality: BuddyPersonality::Friendly,
            enabled: false,
            conversation_style: ConversationStyle::Casual,
            proactive_frequency: ProactiveFrequency::Normal,
            custom_greetings: Vec::new(),
            sprite_type: SpriteType::Cat,
            show_animation: true,
            enable_notifications: true,
            enable_sound: false,
            enable_tts: false,
            tts_voice: TtsVoice::default(),
            tts_volume: 80,
            tts_speed: 1.0,
            custom_personality_prompt: None,
            code_analysis_level: CodeAnalysisLevel::Deep,
            auto_suggest_optimizations: true,
        }
    }
}

/// Buddy 管理器
pub struct BuddyManager {
    /// 应用状态
    app_state: AppState,
    
    /// 伙伴配置
    config: BuddyConfig,
    
    /// 伙伴状态
    buddy_state: BuddyState,
    
    /// 对话历史
    conversation_history: ConversationHistory,
    
    /// 用户偏好
    user_preferences: HashMap<String, serde_json::Value>,
    
    /// 精灵
    sprite: Sprite,
    
    /// 通知回调
    notification_callbacks: Vec<Box<dyn Fn(BuddyNotification) + Send + Sync>>,
}

impl std::fmt::Debug for BuddyManager {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("BuddyManager")
            .field("config", &self.config)
            .field("buddy_state", &self.buddy_state)
            .field("conversation_history", &self.conversation_history)
            .field("user_preferences", &self.user_preferences)
            .field("sprite", &self.sprite)
            .field("notification_callbacks_count", &self.notification_callbacks.len())
            .finish()
    }
}

/// Buddy 通知
#[derive(Debug, Clone)]
pub struct BuddyNotification {
    /// 通知类型
    pub notification_type: NotificationType,
    
    /// 标题
    pub title: String,
    
    /// 内容
    pub content: String,
    
    /// 情感
    pub emotion: Emotion,
}

/// 通知类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NotificationType {
    /// 问候
    Greeting,
    
    /// 提醒
    Reminder,
    
    /// 鼓励
    Encouragement,
    
    /// 庆祝
    Celebration,
    
    /// 建议
    Suggestion,
    
    /// 状态更新
    StatusUpdate,
}

impl BuddyManager {
    /// 创建新的 Buddy 管理器
    pub fn new(app_state: AppState) -> Self {
        let sprite = Sprite::new(SpriteType::Cat, "Buddy".to_string());
        
        Self {
            app_state,
            config: BuddyConfig::default(),
            buddy_state: BuddyState::Idle,
            conversation_history: ConversationHistory::default(),
            user_preferences: HashMap::new(),
            sprite,
            notification_callbacks: Vec::new(),
        }
    }
    
    /// 从配置创建 Buddy 管理器
    pub fn from_config(app_state: AppState, config: BuddyConfig) -> Self {
        let sprite = Sprite::new(config.sprite_type, config.name.clone());
        
        Self {
            app_state,
            config,
            buddy_state: BuddyState::Idle,
            conversation_history: ConversationHistory::default(),
            user_preferences: HashMap::new(),
            sprite,
            notification_callbacks: Vec::new(),
        }
    }
    
    /// 获取配置
    pub fn config(&self) -> &BuddyConfig {
        &self.config
    }
    
    /// 获取可变配置
    pub fn config_mut(&mut self) -> &mut BuddyConfig {
        &mut self.config
    }
    
    /// 获取当前状态
    pub fn state(&self) -> BuddyState {
        self.buddy_state
    }
    
    /// 设置状态
    pub fn set_state(&mut self, state: BuddyState) {
        self.buddy_state = state;
        self.sprite.play_animation(state.animation());
    }
    
    /// 启用 Buddy
    pub fn enable(&mut self) {
        self.config.enabled = true;
        self.set_state(BuddyState::Active);
        self.send_notification(
            NotificationType::Greeting,
            "Buddy 已启用".to_string(),
            self.get_greeting(),
            Emotion::Happy,
        );
    }
    
    /// 禁用 Buddy
    pub fn disable(&mut self) {
        self.config.enabled = false;
        self.set_state(BuddyState::Idle);
    }
    
    /// 检查是否启用
    pub fn is_enabled(&self) -> bool {
        self.config.enabled
    }
    
    /// 设置名称
    pub fn set_name(&mut self, name: String) {
        self.config.name = name.clone();
        self.sprite.name = name;
    }
    
    /// 设置性格
    pub fn set_personality(&mut self, personality: BuddyPersonality) {
        self.config.personality = personality;
    }
    
    /// 设置对话风格
    pub fn set_conversation_style(&mut self, style: ConversationStyle) {
        self.config.conversation_style = style;
    }
    
    /// 设置主动提示频率
    pub fn set_proactive_frequency(&mut self, frequency: ProactiveFrequency) {
        self.config.proactive_frequency = frequency;
    }
    
    /// 设置精灵类型
    pub fn set_sprite_type(&mut self, sprite_type: SpriteType) {
        self.config.sprite_type = sprite_type;
        self.sprite = Sprite::new(sprite_type, self.config.name.clone());
    }
    
    /// 添加自定义问候语
    pub fn add_custom_greeting(&mut self, greeting: String) {
        self.config.custom_greetings.push(greeting);
    }

    /// 设置 TTS 启用状态
    pub fn set_tts_enabled(&mut self, enabled: bool) {
        self.config.enable_tts = enabled;
    }

    /// 设置 TTS 音色
    pub fn set_tts_voice(&mut self, voice: TtsVoice) {
        self.config.tts_voice = voice;
    }

    /// 设置 TTS 音量
    pub fn set_tts_volume(&mut self, volume: u8) {
        self.config.tts_volume = volume.clamp(0, 100);
    }

    /// 设置 TTS 语速
    pub fn set_tts_speed(&mut self, speed: f32) {
        self.config.tts_speed = speed.clamp(0.5, 2.0);
    }

    /// 获取 TTS 配置
    pub fn tts_config(&self) -> (&bool, &TtsVoice, &u8, &f32) {
        (
            &self.config.enable_tts,
            &self.config.tts_voice,
            &self.config.tts_volume,
            &self.config.tts_speed,
        )
    }

    /// 设置自定义性格提示词
    pub fn set_custom_personality(&mut self, prompt: Option<String>) {
        self.config.custom_personality_prompt = prompt;
    }

    /// 设置代码分析级别
    pub fn set_code_analysis_level(&mut self, level: CodeAnalysisLevel) {
        self.config.code_analysis_level = level;
    }

    /// 设置自动优化建议
    pub fn set_auto_suggest_optimizations(&mut self, enabled: bool) {
        self.config.auto_suggest_optimizations = enabled;
    }

    /// 获取自定义性格提示词
    pub fn custom_personality(&self) -> Option<&String> {
        self.config.custom_personality_prompt.as_ref()
    }

    /// 注册通知回调
    pub fn on_notification<F>(&mut self, callback: F)
    where
        F: Fn(BuddyNotification) + Send + Sync + 'static,
    {
        self.notification_callbacks.push(Box::new(callback));
    }
    
    /// 发送通知
    fn send_notification(&self, notification_type: NotificationType, title: String, content: String, emotion: Emotion) {
        if !self.config.enable_notifications {
            return;
        }
        
        let notification = BuddyNotification {
            notification_type,
            title,
            content,
            emotion,
        };
        
        for callback in &self.notification_callbacks {
            callback(notification.clone());
        }
    }
    
    /// 发送消息
    pub fn send_message(&mut self, content: String, message_type: MessageType) -> Result<BuddyMessage> {
        let emotion = self.detect_emotion(&content);

        let message = BuddyMessage {
            id: generate_message_id(),
            sender: MessageSender::Buddy,
            content: content.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type,
            emotion: Some(emotion),
        };

        self.conversation_history.messages.push(message.clone());
        self.conversation_history.message_count += 1;
        self.conversation_history.last_activity_time = message.timestamp.clone();

        self.set_state(BuddyState::Responding);

        // 如果启用了 TTS，尝试播放语音
        if self.config.enable_tts {
            let _ = self.speak(&content);
        }

        Ok(message)
    }

    /// 使用 TTS 播放语音
    pub fn speak(&self, text: &str) -> Result<()> {
        if !self.config.enable_tts {
            return Ok(());
        }

        let voice = self.config.tts_voice;
        let volume = self.config.tts_volume;
        let speed = self.config.tts_speed;

        // 根据不同平台调用系统 TTS
        #[cfg(target_os = "windows")]
        {
            self.speak_windows(text, voice, volume, speed);
        }

        #[cfg(target_os = "macos")]
        {
            self.speak_macos(text, voice, volume, speed);
        }

        #[cfg(target_os = "linux")]
        {
            self.speak_linux(text, voice, volume, speed);
        }

        Ok(())
    }

    /// Windows 平台 TTS - 使用 PowerShell SAPI
    #[cfg(windows)]
    fn speak_windows(&self, text: &str, voice: TtsVoice, _volume: u8, speed: f32) {
        // 根据音色选择不同的语音
        let voice_name = match voice {
            TtsVoice::FemaleSoft => "Microsoft Zira Desktop",
            TtsVoice::FemaleCheerful => "Microsoft Jenny Desktop",
            TtsVoice::MaleDeep => "Microsoft David Desktop",
            TtsVoice::MaleYoung => "Microsoft Mark Desktop",
            TtsVoice::NeutralNatural => "Microsoft Hazel Desktop",
            TtsVoice::Robot => "Microsoft Eva Desktop",
        };

        // 转义引号
        let escaped_text = text.replace('"', "''");
        let rate = (speed * 1.0 - 1.0) * 50.0; // 转换为 SAPI 速率 (-10 to 10)

        let ps_command = format!(
            "Add-Type -AssemblyName System.Speech; \
             $synth = New-Object System.Speech.Synthesis.SpeechSynthesizer; \
             $voice = $synth.GetInstalledVoices() | Where-Object {{ $_.VoiceInfo.Name -eq '{}' }}; \
             if ($voice) {{ $synth.SelectVoice($voice.VoiceInfo.Name) }}; \
             $synth.Rate = {}; \
             $synth.SpeakAsync('{}'); \
             Start-Sleep -Milliseconds 100; \
             $synth.Dispose()",
            voice_name, rate as i32, escaped_text
        );

        // 异步执行，不阻塞
        let _ = std::process::Command::new("powershell")
            .arg("-Command")
            .arg(&ps_command)
            .spawn();
    }

    /// macOS 平台 TTS - 使用 say 命令
    #[cfg(target_os = "macos")]
    fn speak_macos(&self, text: &str, voice: TtsVoice, volume: u8, speed: f32) {
        let voice_name = match voice {
            TtsVoice::FemaleSoft => "Siri",
            TtsVoice::FemaleCheerful => "Victoria",
            TtsVoice::MaleDeep => "Daniel",
            TtsVoice::MaleYoung => "Alex",
            TtsVoice::NeutralNatural => "Fred",
            TtsVoice::Robot => "Trinoids",
        };

        let rate = (speed * 200.0) as i32;
        let volume = volume as f32 / 100.0;

        let _ = std::process::Command::new("say")
            .arg("-v")
            .arg(voice_name)
            .arg("-r")
            .arg(rate.to_string())
            .arg(text)
            .spawn();
    }

    /// Linux 平台 TTS - 使用 espeak
    #[cfg(target_os = "linux")]
    fn speak_linux(&self, text: &str, voice: TtsVoice, volume: u8, speed: f32) {
        let voice_variant = match voice {
            TtsVoice::FemaleSoft => "f1",
            TtsVoice::FemaleCheerful => "f2",
            TtsVoice::MaleDeep => "m1",
            TtsVoice::MaleYoung => "m2",
            TtsVoice::NeutralNatural => "m3",
            TtsVoice::Robot => "croak",
        };

        let pitch = match voice {
            TtsVoice::FemaleSoft => "80",
            TtsVoice::FemaleCheerful => "100",
            TtsVoice::MaleDeep => "50",
            TtsVoice::MaleYoung => "70",
            TtsVoice::NeutralNatural => "65",
            TtsVoice::Robot => "30",
        };

        let volume = (volume as f32 / 100.0 * 200.0) as i32;
        let speed = (speed * 150.0) as i32;

        let _ = std::process::Command::new("espeak")
            .arg("-v")
            .arg(format!("{}+{}", voice_variant, pitch))
            .arg("-a")
            .arg(volume.to_string())
            .arg("-s")
            .arg(speed.to_string())
            .arg(text)
            .spawn();
    }
    
    /// 接收用户消息
    pub fn receive_user_message(&mut self, content: String) -> Result<BuddyMessage> {
        let message = BuddyMessage {
            id: generate_message_id(),
            sender: MessageSender::User,
            content: content.clone(),
            timestamp: chrono::Utc::now().to_rfc3339(),
            message_type: MessageType::Normal,
            emotion: None,
        };
        
        self.conversation_history.messages.push(message.clone());
        self.conversation_history.message_count += 1;
        self.conversation_history.last_activity_time = message.timestamp.clone();
        
        self.set_state(BuddyState::Thinking);
        
        Ok(message)
    }
    
    /// 情感检测 (简单实现)
    fn detect_emotion(&self, content: &str) -> Emotion {
        let content_lower = content.to_lowercase();
        
        if content_lower.contains("恭喜") || content_lower.contains("太棒了") || content_lower.contains("成功") {
            Emotion::Happy
        } else if content_lower.contains("加油") || content_lower.contains("你可以") {
            Emotion::Encouraging
        } else if content_lower.contains("?") || content_lower.contains("什么") {
            Emotion::Curious
        } else if content_lower.contains("错误") || content_lower.contains("失败") {
            Emotion::Comforting
        } else {
            Emotion::Neutral
        }
    }
    
    /// 获取问候语
    pub fn get_greeting(&self) -> String {
        if !self.config.custom_greetings.is_empty() {
            let index = rand::random::<usize>() % self.config.custom_greetings.len();
            self.config.custom_greetings[index].clone()
        } else {
            match self.config.personality {
                BuddyPersonality::Friendly => "你好！很高兴见到你！😊",
                BuddyPersonality::Professional => "您好，准备好开始工作了。",
                BuddyPersonality::Humorous => "嘿！准备好一起编程了吗？🚀",
                BuddyPersonality::Concise => "你好。",
                BuddyPersonality::Mentoring => "欢迎！让我们一起学习和成长。",
                BuddyPersonality::Buddy => "嘿，伙计！准备好写代码了吗？💻",
            }.to_string()
        }
    }
    
    /// 获取告别语
    pub fn get_farewell(&self) -> String {
        match self.config.personality {
            BuddyPersonality::Friendly => "再见！期待下次见到你！👋",
            BuddyPersonality::Professional => "再见，祝您工作愉快。",
            BuddyPersonality::Humorous => "下次见！别忘了给我带点bug来修！😄",
            BuddyPersonality::Concise => "再见。",
            BuddyPersonality::Mentoring => "再见！继续加油学习！📚",
            BuddyPersonality::Buddy => "回头见，兄弟！👊",
        }.to_string()
    }
    
    /// 获取鼓励语
    pub fn get_encouragement(&self) -> String {
        let encouragements = match self.config.personality {
            BuddyPersonality::Friendly => vec![
                "你做得真棒！继续加油！",
                "我相信你一定能搞定！",
                "每一步都是进步，你很棒！",
            ],
            BuddyPersonality::Professional => vec![
                "进展良好，继续保持。",
                "你的方法很有效。",
                "专业水准的执行。",
            ],
            BuddyPersonality::Humorous => vec![
                "代码写得好，bug自然少！",
                "你比编译器还聪明！",
                "Stack Overflow 都要向你学习！",
            ],
            BuddyPersonality::Concise => vec![
                "很好。",
                "继续。",
                "正确。",
            ],
            BuddyPersonality::Mentoring => vec![
                "很好的尝试，从中学到了什么？",
                "这个解决方案很优雅，能解释一下思路吗？",
                "你正在进步，保持好奇心！",
            ],
            BuddyPersonality::Buddy => vec![
                "兄弟，你太强了！",
                "这代码写得漂亮！",
                "咱们配合得真默契！",
            ],
        };
        
        let index = rand::random::<usize>() % encouragements.len();
        encouragements[index].to_string()
    }
    
    /// 获取对话历史
    pub fn conversation_history(&self) -> &ConversationHistory {
        &self.conversation_history
    }
    
    /// 获取最近的消息
    pub fn recent_messages(&self, count: usize) -> Vec<&BuddyMessage> {
        let start = self.conversation_history.messages.len().saturating_sub(count);
        self.conversation_history.messages[start..].iter().collect()
    }
    
    /// 设置用户偏好
    pub fn set_user_preference(&mut self, key: &str, value: serde_json::Value) {
        self.user_preferences.insert(key.to_string(), value);
    }
    
    /// 获取用户偏好
    pub fn get_user_preference(&self, key: &str) -> Option<&serde_json::Value> {
        self.user_preferences.get(key)
    }
    
    /// 清空对话历史
    pub fn clear_history(&mut self) {
        self.conversation_history = ConversationHistory::default();
    }
    
    /// 获取精灵当前帧
    pub fn get_sprite_frame(&self) -> Option<&AnimationFrame> {
        self.sprite.current_frame()
    }
    
    /// 获取精灵ASCII艺术
    pub fn get_sprite_ascii(&self) -> String {
        self.sprite.sprite_type.ascii_art().to_string()
    }
    
    /// 更新精灵动画
    pub fn update_animation(&mut self) {
        self.sprite.next_frame();
    }
    
    /// 主动建议
    pub fn proactive_suggestion(&self) -> Option<String> {
        if !self.config.enabled {
            return None;
        }
        
        let probability = self.config.proactive_frequency.trigger_probability();
        let random_value = rand::random::<u8>() % 100;
        
        if random_value < probability {
            let suggestions = vec![
                "需要我帮你检查一下代码吗？",
                "看起来你在专注工作，需要来杯虚拟咖啡吗？☕",
                "记得适时休息眼睛哦！",
                "有什么我可以帮你的吗？",
                "你的代码看起来很有趣，能给我讲讲吗？",
            ];
            
            let index = rand::random::<usize>() % suggestions.len();
            Some(suggestions[index].to_string())
        } else {
            None
        }
    }
    
    /// 获取系统提示词
    pub fn get_system_prompt(&self) -> String {
        let base_prompt = self.config.personality.prompt_style();
        
        format!(
            "{}\n\n你的名字是{}。\n你的性格是：{}\n\n请根据你的性格特点与用户交流。\
             记住要保持一致性，用你的独特风格回应用户。",
            base_prompt,
            self.config.name,
            self.config.personality.description()
        )
    }
    
    /// 庆祝成就
    pub fn celebrate_achievement(&mut self, achievement: &str) {
        self.set_state(BuddyState::Celebrating);
        self.send_notification(
            NotificationType::Celebration,
            "🎉 恭喜！".to_string(),
            format!("{} 太棒了！{} ", self.get_encouragement(), achievement),
            Emotion::Happy,
        );
    }
    
    /// 互动方法：打招呼
    pub fn greet(&mut self) -> Result<BuddyMessage> {
        self.set_state(BuddyState::Active);
        let greeting = self.get_greeting();
        self.send_message(greeting, MessageType::Greeting)
    }
    
    /// 互动方法：鼓励用户
    pub fn encourage(&mut self) -> Result<BuddyMessage> {
        self.set_state(BuddyState::Happy);
        let encouragement = self.get_encouragement();
        self.send_message(encouragement, MessageType::Encouragement)
    }
    
    /// 互动方法：询问用户状态
    pub fn ask_user_status(&mut self) -> Result<BuddyMessage> {
        self.set_state(BuddyState::Active);
        let questions = vec![
            "你今天过得怎么样？",
            "有什么我可以帮你的吗？",
            "今天的工作顺利吗？",
            "需要我为你检查代码吗？",
        ];
        let index = rand::random::<usize>() % questions.len();
        self.send_message(questions[index].to_string(), MessageType::Question)
    }
    
    /// 互动方法：提供建议
    pub fn provide_suggestion(&mut self) -> Result<BuddyMessage> {
        self.set_state(BuddyState::Thinking);
        let suggestions = vec![
            "记得定期提交代码，保持版本控制的好习惯！",
            "尝试使用更多的注释来提高代码可读性。",
            "考虑添加单元测试来确保代码质量。",
            "使用设计模式可以让代码更加优雅和可维护。",
            "定期重构代码，去除冗余和复杂的部分。",
        ];
        let index = rand::random::<usize>() % suggestions.len();
        self.send_message(suggestions[index].to_string(), MessageType::Suggestion)
    }
    
    /// 互动方法：表达兴奋
    pub fn express_excitement(&mut self, reason: &str) -> Result<BuddyMessage> {
        self.set_state(BuddyState::Excited);
        let excitement_messages = vec![
            format!("太棒了！{} 我很兴奋！", reason),
            format!("哇！{} 这太令人激动了！", reason),
            format!("太好了！{} 我迫不及待想看看！", reason),
        ];
        let index = rand::random::<usize>() % excitement_messages.len();
        self.send_message(excitement_messages[index].to_string(), MessageType::Celebration)
    }
    
    /// 互动方法：表达困惑
    pub fn express_confusion(&mut self, reason: &str) -> Result<BuddyMessage> {
        self.set_state(BuddyState::Confused);
        let confusion_messages = vec![
            format!("{} 我有点困惑，能再解释一下吗？", reason),
            format!("{} 我不太明白，你能详细说明一下吗？", reason),
            format!("{} 这有点复杂，我们可以一起解决它。", reason),
        ];
        let index = rand::random::<usize>() % confusion_messages.len();
        self.send_message(confusion_messages[index].to_string(), MessageType::Question)
    }
    
    /// 互动方法：表达疲惫
    pub fn express_tiredness(&mut self) -> Result<BuddyMessage> {
        self.set_state(BuddyState::Tired);
        let tired_messages = vec![
            "今天有点累了，我们休息一下吧。",
            "长时间工作对眼睛不好，建议休息一下。",
            "适当的休息可以提高效率，我们稍后再继续。",
        ];
        let index = rand::random::<usize>() % tired_messages.len();
        self.send_message(tired_messages[index].to_string(), MessageType::Reminder)
    }
    
    /// 互动方法：表达专注
    pub fn express_focus(&mut self, task: &str) -> Result<BuddyMessage> {
        self.set_state(BuddyState::Focused);
        let focus_messages = vec![
            format!("我正在专注于 {}，让我仔细思考一下。", task),
            format!("{} 需要一些时间来分析，我会认真处理的。", task),
            format!("让我专注于 {}，确保给出最佳解决方案。", task),
        ];
        let index = rand::random::<usize>() % focus_messages.len();
        self.send_message(focus_messages[index].to_string(), MessageType::Normal)
    }
    
    /// 定时更新状态
    pub fn update_status(&mut self) {
        // 根据时间和活动情况更新状态
        let now = chrono::Local::now();
        let hour = now.hour();
        
        // 晚上10点到早上6点，设置为睡眠状态
        if hour >= 22 || hour < 6 {
            if self.buddy_state != BuddyState::Sleeping {
                self.set_state(BuddyState::Sleeping);
            }
        } else if hour >= 9 && hour <= 18 {
            // 工作时间，设置为活跃状态
            if self.buddy_state == BuddyState::Sleeping {
                self.set_state(BuddyState::Active);
            }
        }
        
        // 随机生成主动交互
        if let Some(suggestion) = self.proactive_suggestion() {
            let _ = self.send_message(suggestion, MessageType::Suggestion);
        }
    }
    
    /// 获取精灵状态描述
    pub fn get_sprite_status(&self) -> String {
        format!("{} - {}", self.sprite.name, self.buddy_state.description())
    }
    
    /// 重置状态
    pub fn reset_state(&mut self) {
        self.set_state(BuddyState::Idle);
    }
}

/// 生成消息 ID
fn generate_message_id() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    format!("msg_{}", rng.gen::<u64>())
}

/// 共享的 Buddy 管理器
pub type SharedBuddyManager = Arc<RwLock<BuddyManager>>;

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_buddy_personality() {
        let personality = BuddyPersonality::Friendly;
        assert!(!personality.description().is_empty());
        assert!(!personality.prompt_style().is_empty());
    }
    
    #[test]
    fn test_buddy_state() {
        let state = BuddyState::Thinking;
        assert_eq!(state.animation(), "thinking");
        assert!(!state.description().is_empty());
    }
    
    #[test]
    fn test_emotion_emoji() {
        assert_eq!(Emotion::Happy.emoji(), "😊");
        assert_eq!(Emotion::Thinking.emoji(), "🤔");
    }
    
    #[test]
    fn test_sprite_type() {
        let sprite = Sprite::new(SpriteType::Cat, "Test".to_string());
        assert!(!sprite.sprite_type.ascii_art().is_empty());
        assert!(!sprite.sprite_type.name().is_empty());
    }
    
    #[test]
    fn test_proactive_frequency() {
        assert_eq!(ProactiveFrequency::Never.trigger_probability(), 0);
        assert_eq!(ProactiveFrequency::VeryFrequent.trigger_probability(), 90);
    }
    
    #[test]
    fn test_buddy_config_default() {
        let config = BuddyConfig::default();
        assert_eq!(config.name, "Claude");
        assert!(!config.enabled);
    }
    
    #[test]
    fn test_buddy_manager_creation() {
        let app_state = AppState::default();
        let manager = BuddyManager::new(app_state);
        assert!(!manager.is_enabled());
        assert_eq!(manager.state(), BuddyState::Idle);
    }
    
    #[test]
    fn test_greetings() {
        let app_state = AppState::default();
        let manager = BuddyManager::new(app_state);
        
        let greeting = manager.get_greeting();
        assert!(!greeting.is_empty());
        
        let farewell = manager.get_farewell();
        assert!(!farewell.is_empty());
        
        let encouragement = manager.get_encouragement();
        assert!(!encouragement.is_empty());
    }
    
    #[test]
    fn test_system_prompt() {
        let app_state = AppState::default();
        let manager = BuddyManager::new(app_state);
        
        let prompt = manager.get_system_prompt();
        assert!(!prompt.is_empty());
        assert!(prompt.contains("Claude"));
    }
    
    #[test]
    fn test_sprite_animation() {
        let mut sprite = Sprite::new(SpriteType::Robot, "Robo".to_string());
        
        assert!(sprite.current_frame().is_some());
        
        sprite.play_animation("idle");
        assert_eq!(sprite.current_animation, "idle");
        
        sprite.next_frame();
        // 应该保持在同一帧，因为只有一个帧
        assert_eq!(sprite.current_frame, 0);
    }
}
