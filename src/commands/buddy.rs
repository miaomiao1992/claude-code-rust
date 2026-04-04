//! Buddy 伴侣精灵命令
//!
//! 提供命令行接口来管理 Buddy 伴侣精灵系统

use crate::commands::registry::CommandExecutor as CmdExecutor;
use crate::commands::types::{Command, CommandBase, CommandContext, CommandResult, LocalCommand, LoadedFrom};
use crate::error::Result;
use crate::features::buddy::{
    BuddyConfig, BuddyManager, BuddyPersonality, ConversationStyle, ProactiveFrequency, SpriteType, TtsVoice,
};
use crate::state::AppState;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Buddy 命令执行器
pub struct BuddyCommandExecutor {
    buddy_manager: Arc<RwLock<BuddyManager>>,
}

impl BuddyCommandExecutor {
    /// 创建新的 Buddy 命令执行器
    pub fn new(app_state: AppState) -> Self {
        let buddy_manager = Arc::new(RwLock::new(BuddyManager::new(app_state)));
        Self { buddy_manager }
    }

    /// 从配置创建
    pub fn from_config(app_state: AppState, config: BuddyConfig) -> Self {
        let buddy_manager = Arc::new(RwLock::new(BuddyManager::from_config(app_state, config)));
        Self { buddy_manager }
    }

    /// 获取 Buddy 管理器
    pub fn buddy_manager(&self) -> Arc<RwLock<BuddyManager>> {
        self.buddy_manager.clone()
    }

    /// 解析子命令
    fn parse_subcommand(&self, args: &[String]) -> Option<BuddySubcommand> {
        if args.is_empty() {
            return None;
        }

        match args[0].as_str() {
            "enable" => Some(BuddySubcommand::Enable),
            "disable" => Some(BuddySubcommand::Disable),
            "status" => Some(BuddySubcommand::Status),
            "show" => Some(BuddySubcommand::Show),
            "greet" => Some(BuddySubcommand::Greet),
            "encourage" => Some(BuddySubcommand::Encourage),
            "clear" => Some(BuddySubcommand::Clear),
            "configure" => {
                let mut config = ConfigureOptions::default();
                let mut i = 1;
                while i < args.len() {
                    match args[i].as_str() {
                        "--name" | "-n" => {
                            if i + 1 < args.len() {
                                config.name = Some(args[i + 1].clone());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "--personality" | "-p" => {
                            if i + 1 < args.len() {
                                config.personality = Some(args[i + 1].clone());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "--style" | "-s" => {
                            if i + 1 < args.len() {
                                config.style = Some(args[i + 1].clone());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "--frequency" | "-f" => {
                            if i + 1 < args.len() {
                                config.frequency = Some(args[i + 1].clone());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "--sprite" => {
                            if i + 1 < args.len() {
                                config.sprite = Some(args[i + 1].clone());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "--animation" => {
                            if i + 1 < args.len() {
                                config.animation = args[i + 1].parse().ok();
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "--notifications" => {
                            if i + 1 < args.len() {
                                config.notifications = args[i + 1].parse().ok();
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "--tts" => {
                            if i + 1 < args.len() {
                                config.tts = args[i + 1].parse().ok();
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "--voice" => {
                            if i + 1 < args.len() {
                                config.voice = Some(args[i + 1].clone());
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "--volume" => {
                            if i + 1 < args.len() {
                                config.volume = args[i + 1].parse().ok();
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        "--speed" => {
                            if i + 1 < args.len() {
                                config.speed = args[i + 1].parse().ok();
                                i += 2;
                            } else {
                                i += 1;
                            }
                        }
                        _ => i += 1,
                    }
                }
                Some(BuddySubcommand::Configure(config))
            }
            "chat" => {
                if args.len() > 1 {
                    Some(BuddySubcommand::Chat(args[1..].join(" ")))
                } else {
                    Some(BuddySubcommand::Chat("".to_string()))
                }
            }
            "export" => {
                let mut output = None;
                let mut i = 1;
                while i < args.len() {
                    if args[i] == "--output" || args[i] == "-o" {
                        if i + 1 < args.len() {
                            output = Some(args[i + 1].clone());
                            i += 2;
                        } else {
                            i += 1;
                        }
                    } else {
                        i += 1;
                    }
                }
                Some(BuddySubcommand::Export(output))
            }
            "import" => {
                if args.len() > 1 {
                    Some(BuddySubcommand::Import(args[1].clone()))
                } else {
                    Some(BuddySubcommand::Import("".to_string()))
                }
            }
            "list-personalities" => Some(BuddySubcommand::ListPersonalities),
            "list-sprites" => Some(BuddySubcommand::ListSprites),
            "list-voices" => Some(BuddySubcommand::ListVoices),
            _ => None,
        }
    }
}

/// Buddy 子命令
#[derive(Debug)]
enum BuddySubcommand {
    Enable,
    Disable,
    Status,
    Show,
    Greet,
    Encourage,
    Clear,
    Configure(ConfigureOptions),
    Chat(String),
    Export(Option<String>),
    Import(String),
    ListPersonalities,
    ListSprites,
    ListVoices,
}

/// 配置选项
#[derive(Debug, Default)]
struct ConfigureOptions {
    name: Option<String>,
    personality: Option<String>,
    style: Option<String>,
    frequency: Option<String>,
    sprite: Option<String>,
    animation: Option<bool>,
    notifications: Option<bool>,
    tts: Option<bool>,
    voice: Option<String>,
    volume: Option<u8>,
    speed: Option<f32>,
}

#[async_trait::async_trait]
impl CmdExecutor for BuddyCommandExecutor {
    async fn execute(&self, context: CommandContext) -> Result<CommandResult> {
        let args = context.args.clone();
        let args_vec: Vec<String> = if args.is_empty() {
            vec![]
        } else {
            args.split_whitespace().map(|s| s.to_string()).collect()
        };
        
        let subcommand = match self.parse_subcommand(&args_vec) {
            Some(cmd) => cmd,
            None => {
                return Ok(CommandResult {
                    content: self.get_help_text(),
                    ..Default::default()
                });
            }
        };

        let mut manager = self.buddy_manager.write().await;

        match subcommand {
            BuddySubcommand::Enable => {
                manager.enable();
                let name = manager.config().name.clone();
                drop(manager);

                let sprite = self.buddy_manager.read().await.get_sprite_ascii();
                Ok(CommandResult {
                    content: format!(
                        "{}\n✨ {} 已启用！{}\n",
                        sprite,
                        name,
                        self.buddy_manager.read().await.get_greeting()
                    ),
                    ..Default::default()
                })
            }

            BuddySubcommand::Disable => {
                let farewell = manager.get_farewell();
                manager.disable();
                Ok(CommandResult {
                    content: format!("👋 Buddy 已禁用\n{}", farewell),
                    ..Default::default()
                })
            }

            BuddySubcommand::Status => {
                let config = manager.config();
                let state = manager.state();
                let history = manager.conversation_history();

                let status_text = format!(
                    "🤖 Buddy 状态\n\
                     ─────────────\n\
                     启用状态: {}\n\
                     当前状态: {}\n\
                     名称: {}\n\
                     性格: {}\n\
                     对话风格: {:?}\n\
                     主动频率: {:?}\n\
                     精灵类型: {}\n\
                     动画显示: {}\n\
                     通知: {}\n\
                     TTS语音: {}\n\
                     TTS音色: {}\n\
                     TTS音量: {}%\n\
                     TTS语速: {:.1}x\n\
                     对话消息数: {}\n                    ",
                    if config.enabled { "✅ 已启用" } else { "❌ 已禁用" },
                    state.description(),
                    config.name,
                    config.personality.description(),
                    config.conversation_style,
                    config.proactive_frequency,
                    config.sprite_type.name(),
                    if config.show_animation { "开启" } else { "关闭" },
                    if config.enable_notifications { "开启" } else { "关闭" },
                    if config.enable_tts { "✅ 已开启" } else { "❌ 已关闭" },
                    config.tts_voice.display_name(),
                    config.tts_volume,
                    config.tts_speed,
                    history.message_count
                );

                Ok(CommandResult {
                    content: status_text,
                    ..Default::default()
                })
            }

            BuddySubcommand::Configure(options) => {
                let mut changes = Vec::new();

                if let Some(n) = options.name {
                    manager.set_name(n.clone());
                    changes.push(format!("名称: {}", n));
                }

                if let Some(p) = options.personality {
                    let personality_type = match p.to_lowercase().as_str() {
                        "friendly" => BuddyPersonality::Friendly,
                        "professional" => BuddyPersonality::Professional,
                        "humorous" => BuddyPersonality::Humorous,
                        "concise" => BuddyPersonality::Concise,
                        "mentoring" => BuddyPersonality::Mentoring,
                        "buddy" => BuddyPersonality::Buddy,
                        _ => {
                            return Ok(CommandResult {
                                content: format!(
                                    "未知的性格类型: {}。可用类型: friendly, professional, humorous, concise, mentoring, buddy",
                                    p
                                ),
                                ..Default::default()
                            });
                        }
                    };
                    manager.set_personality(personality_type);
                    changes.push(format!("性格: {}", personality_type.description()));
                }

                if let Some(s) = options.style {
                    let style_type = match s.to_lowercase().as_str() {
                        "formal" => ConversationStyle::Formal,
                        "casual" => ConversationStyle::Casual,
                        "semiformal" => ConversationStyle::SemiFormal,
                        _ => {
                            return Ok(CommandResult {
                                content: format!(
                                    "未知的对话风格: {}。可用风格: formal, casual, semiformal",
                                    s
                                ),
                                ..Default::default()
                            });
                        }
                    };
                    manager.set_conversation_style(style_type);
                    changes.push(format!("对话风格: {:?}", style_type));
                }

                if let Some(f) = options.frequency {
                    let freq_type = match f.to_lowercase().as_str() {
                        "never" => ProactiveFrequency::Never,
                        "rare" => ProactiveFrequency::Rare,
                        "normal" => ProactiveFrequency::Normal,
                        "frequent" => ProactiveFrequency::Frequent,
                        "veryfrequent" => ProactiveFrequency::VeryFrequent,
                        _ => {
                            return Ok(CommandResult {
                                content: format!(
                                    "未知的频率: {}。可用频率: never, rare, normal, frequent, veryfrequent",
                                    f
                                ),
                                ..Default::default()
                            });
                        }
                    };
                    manager.set_proactive_frequency(freq_type);
                    changes.push(format!("主动频率: {:?}", freq_type));
                }

                if let Some(s) = options.sprite {
                    let sprite_type = match s.to_lowercase().as_str() {
                        "cat" => SpriteType::Cat,
                        "dog" => SpriteType::Dog,
                        "robot" => SpriteType::Robot,
                        "alien" => SpriteType::Alien,
                        "ghost" => SpriteType::Ghost,
                        "golden" => SpriteType::Golden,
                        "gold" => SpriteType::Golden,
                        _ => {
                            return Ok(CommandResult {
                                content: format!(
                                    "未知的精灵类型: {}。可用类型: cat, dog, robot, alien, ghost, golden",
                                    s
                                ),
                                ..Default::default()
                            });
                        }
                    };
                    manager.set_sprite_type(sprite_type);
                    changes.push(format!("精灵类型: {}", sprite_type.name()));
                }

                if let Some(a) = options.animation {
                    manager.config_mut().show_animation = a;
                    changes.push(format!("动画显示: {}", if a { "开启" } else { "关闭" }));
                }

                if let Some(n) = options.notifications {
                    manager.config_mut().enable_notifications = n;
                    changes.push(format!("通知: {}", if n { "开启" } else { "关闭" }));
                }

                if let Some(t) = options.tts {
                    manager.set_tts_enabled(t);
                    changes.push(format!("TTS语音: {}", if t { "开启" } else { "关闭" }));
                }

                if let Some(v) = options.voice {
                    let voice_type = match v.to_lowercase().as_str() {
                        "female_soft" | "soft" => TtsVoice::FemaleSoft,
                        "female_cheerful" | "cheerful" => TtsVoice::FemaleCheerful,
                        "male_deep" | "deep" => TtsVoice::MaleDeep,
                        "male_young" | "young" => TtsVoice::MaleYoung,
                        "neutral" | "natural" => TtsVoice::NeutralNatural,
                        "robot" => TtsVoice::Robot,
                        _ => {
                            return Ok(CommandResult {
                                content: format!(
                                    "未知的音色类型: {}。可用类型: female_soft, female_cheerful, male_deep, male_young, neutral, robot",
                                    v
                                ),
                                ..Default::default()
                            });
                        }
                    };
                    manager.set_tts_voice(voice_type);
                    changes.push(format!("TTS音色: {}", voice_type.display_name()));
                }

                if let Some(v) = options.volume {
                    manager.set_tts_volume(v);
                    changes.push(format!("TTS音量: {}%", v));
                }

                if let Some(s) = options.speed {
                    manager.set_tts_speed(s);
                    changes.push(format!("TTS语速: {:.1}x", s));
                }

                if changes.is_empty() {
                    Ok(CommandResult {
                        content: "没有进行任何更改。使用 buddy configure --help 查看可用选项。".to_string(),
                        ..Default::default()
                    })
                } else {
                    Ok(CommandResult {
                        content: format!("✅ 配置已更新:\n{}", changes.join("\n")),
                        ..Default::default()
                    })
                }
            }

            BuddySubcommand::Show => {
                if !manager.is_enabled() {
                    return Ok(CommandResult {
                        content: "Buddy 当前未启用。使用 'buddy enable' 启用。".to_string(),
                        ..Default::default()
                    });
                }

                let sprite = manager.get_sprite_ascii();
                let state = manager.state();

                Ok(CommandResult {
                    content: format!("{}\n状态: {}", sprite, state.description()),
                    ..Default::default()
                })
            }

            BuddySubcommand::Chat(message) => {
                if !manager.is_enabled() {
                    return Ok(CommandResult {
                        content: "Buddy 当前未启用。使用 'buddy enable' 启用。".to_string(),
                        ..Default::default()
                    });
                }

                if message.is_empty() {
                    return Ok(CommandResult {
                        content: "请提供消息内容。用法: buddy chat <消息>".to_string(),
                        ..Default::default()
                    });
                }

                // 接收用户消息
                manager.receive_user_message(message.clone())?;

                // 生成回复（这里简化处理，实际应该调用AI）
                let response = format!(
                    "{} 收到了你的消息: '{}'",
                    manager.config().name,
                    message
                );

                manager.send_message(response.clone(), crate::features::buddy::MessageType::Normal)?;

                Ok(CommandResult {
                    content: format!(
                        "👤 你: {}\n🤖 {}: {}",
                        message,
                        manager.config().name,
                        response
                    ),
                    ..Default::default()
                })
            }

            BuddySubcommand::Greet => {
                let greeting = manager.get_greeting();
                let sprite = manager.get_sprite_ascii();

                Ok(CommandResult {
                    content: format!("{}\n{}", sprite, greeting),
                    ..Default::default()
                })
            }

            BuddySubcommand::Encourage => {
                let encouragement = manager.get_encouragement();
                let sprite = manager.get_sprite_ascii();

                Ok(CommandResult {
                    content: format!("{}\n💪 {}", sprite, encouragement),
                    ..Default::default()
                })
            }

            BuddySubcommand::Clear => {
                let msg_count = manager.conversation_history().message_count;
                manager.clear_history();
                Ok(CommandResult {
                    content: format!("🗑️ 已清空对话历史（{} 条消息）", msg_count),
                    ..Default::default()
                })
            }

            BuddySubcommand::Export(output) => {
                let config = manager.config().clone();
                let json = serde_json::to_string_pretty(&config)?;

                if let Some(path) = output {
                    tokio::fs::write(&path, &json).await?;
                    Ok(CommandResult {
                        content: format!("✅ 配置已导出到: {}", path),
                        ..Default::default()
                    })
                } else {
                    Ok(CommandResult {
                        content: format!("📋 Buddy 配置:\n{}", json),
                        ..Default::default()
                    })
                }
            }

            BuddySubcommand::Import(path) => {
                if path.is_empty() {
                    return Ok(CommandResult {
                        content: "请提供配置文件路径。用法: buddy import <路径>".to_string(),
                        ..Default::default()
                    });
                }

                let content = tokio::fs::read_to_string(&path).await?;
                let config: BuddyConfig = serde_json::from_str(&content)?;

                // 更新配置
                *manager.config_mut() = config;

                Ok(CommandResult {
                    content: format!("✅ 配置已从 {} 导入", path),
                    ..Default::default()
                })
            }

            BuddySubcommand::ListPersonalities => {
                let personalities = vec![
                    ("friendly", "友好型", "友好热情，总是乐于帮助"),
                    ("professional", "专业型", "专业严谨，注重效率"),
                    ("humorous", "幽默型", "幽默风趣，让编程更有趣"),
                    ("concise", "简洁型", "简洁直接，不拖泥带水"),
                    ("mentoring", "导师型", "耐心指导，帮助你成长"),
                    ("buddy", "伙伴型", "像老朋友一样，轻松自在"),
                ];

                let mut output = String::from("🎭 可用性格类型:\n\n");
                for (id, name, desc) in personalities {
                    output.push_str(&format!("  {} - {}\n    {}\n\n", id, name, desc));
                }

                Ok(CommandResult {
                    content: output,
                    ..Default::default()
                })
            }

            BuddySubcommand::ListSprites => {
                let sprites = vec![
                    ("cat", "小猫咪", SpriteType::Cat),
                    ("dog", "小狗狗", SpriteType::Dog),
                    ("robot", "机器人", SpriteType::Robot),
                    ("alien", "外星人", SpriteType::Alien),
                    ("ghost", "小幽灵", SpriteType::Ghost),
                    ("golden", "金色传说 ✨", SpriteType::Golden),
                ];

                let mut output = String::from("🎨 可用精灵类型:\n\n");
                for (id, name, sprite_type) in sprites {
                    output.push_str(&format!("  {} - {}\n{}\n", id, name, sprite_type.ascii_art()));
                }

                Ok(CommandResult {
                    content: output,
                    ..Default::default()
                })
            }

            BuddySubcommand::ListVoices => {
                let voices = vec![
                    ("female_soft (soft)", "女声-温柔", TtsVoice::FemaleSoft),
                    ("female_cheerful (cheerful)", "女声-活泼", TtsVoice::FemaleCheerful),
                    ("male_deep (deep)", "男声-沉稳", TtsVoice::MaleDeep),
                    ("male_young (young)", "男声-年轻", TtsVoice::MaleYoung),
                    ("neutral (natural)", "中性-自然", TtsVoice::NeutralNatural),
                    ("robot", "机器人", TtsVoice::Robot),
                ];

                let mut output = String::from("🔊 可用 TTS 音色:\n\n");
                for (id, name, _voice_type) in voices {
                    output.push_str(&format!("  {} - {}\n", id, name));
                }
                output.push_str("\n示例: /buddy configure --voice female_soft");

                Ok(CommandResult {
                    content: output,
                    ..Default::default()
                })
            }
        }
    }

    fn command(&self) -> Command {
        Command::Local(LocalCommand {
            base: CommandBase {
                name: "buddy".to_string(),
                description: "管理 Buddy 伴侣精灵系统".to_string(),
                has_user_specified_description: Some(true),
                aliases: Some(vec!["pet".to_string(), "companion".to_string()]),
                availability: None,
                is_hidden: Some(false),
                is_mcp: Some(false),
                argument_hint: Some("<子命令> [选项]".to_string()),
                when_to_use: Some("当你想要与 AI 伙伴交互或配置伴侣精灵时使用".to_string()),
                version: None,
                disable_model_invocation: Some(false),
                user_invocable: Some(true),
                loaded_from: Some(LoadedFrom::Bundled),
                kind: None,
                immediate: Some(false),
                is_sensitive: Some(false),
            },
            supports_non_interactive: true,
        })
    }
}

impl BuddyCommandExecutor {
    /// 获取帮助文本
    fn get_help_text(&self) -> String {
        r#"🤖 Buddy 伴侣精灵系统

用法: /buddy <子命令> [选项]

子命令:
  enable                    启用 Buddy
  disable                   禁用 Buddy
  status                    查看 Buddy 状态
  configure [选项]          配置 Buddy
    --name, -n <名称>       设置名称
    --personality, -p <类型> 设置性格 (friendly, professional, humorous, concise, mentoring, buddy)
    --style, -s <风格>      设置对话风格 (formal, casual, semiformal)
    --frequency, -f <频率>  设置主动提示频率 (never, rare, normal, frequent, veryfrequent)
    --sprite <类型>         设置精灵类型 (cat, dog, robot, alien, ghost, golden)
    --animation <true/false> 启用/禁用动画
    --notifications <true/false> 启用/禁用通知
    --tts <true/false>      启用/禁用 TTS 语音
    --voice <音色>          设置 TTS 音色 (female_soft, female_cheerful, male_deep, male_young, neutral, robot)
    --volume <0-100>        设置 TTS 音量 (默认 80)
    --speed <0.5-2.0>       设置 TTS 语速 (默认 1.0)
  show                      显示 Buddy 精灵
  chat <消息>               与 Buddy 对话
  greet                     显示问候语
  encourage                 获取鼓励
  clear                     清空对话历史
  export [--output <路径>]  导出配置
  import <路径>             导入配置
  list-personalities        列出性格类型
  list-sprites              列出精灵类型
  list-voices               列出 TTS 音色

示例:
  /buddy enable
  /buddy configure --name "小助手" --personality friendly
  /buddy chat "你好"
  /buddy show
"#.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_buddy_command_executor_creation() {
        let app_state = AppState::default();
        let executor = BuddyCommandExecutor::new(app_state);
        assert_eq!(executor.command().name(), "buddy");
    }

    #[test]
    fn test_parse_subcommand() {
        let app_state = AppState::default();
        let executor = BuddyCommandExecutor::new(app_state);

        // 测试 enable
        let args = vec!["enable".to_string()];
        assert!(matches!(executor.parse_subcommand(&args), Some(BuddySubcommand::Enable)));

        // 测试 status
        let args = vec!["status".to_string()];
        assert!(matches!(executor.parse_subcommand(&args), Some(BuddySubcommand::Status)));

        // 测试 chat
        let args = vec!["chat".to_string(), "hello".to_string()];
        if let Some(BuddySubcommand::Chat(msg)) = executor.parse_subcommand(&args) {
            assert_eq!(msg, "hello");
        } else {
            panic!("Expected Chat subcommand");
        }
    }
}
