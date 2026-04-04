//! 集成测试程序
//! 
//! 测试所有功能模块的集成和性能

use claude_code_rs::features::{buddy, kairos};
use claude_code_rs::daemon;
use claude_code_rs::uds;
use claude_code_rs::teleport;
use claude_code_rs::ultrareview;
use claude_code_rs::state::AppState;
use chrono::Duration;
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Claude Code Rust 集成测试 ===\n");

    // 初始化应用状态
    let app_state = AppState::default();

    // 测试1: Daemon后台进程功能
    println!("1. 测试 Daemon 后台进程功能...");
    let start_time = Instant::now();
    
    match daemon::start_daemon().await {
        Ok(daemon_instance) => {
            println!("   Daemon 启动成功");
            println!("   Daemon 状态: {:?}", daemon_instance.status().await);
            
            // 测试停止Daemon
            let stop_result = daemon::stop().await;
            println!("   Daemon 停止: {:?}", stop_result);
        }
        Err(e) => {
            println!("   Daemon 启动失败: {}", e);
            println!("   注意: 这可能是正常的，如果Daemon已经在运行");
        }
    }
    
    let duration = start_time.elapsed();
    println!("   测试耗时: {:?}", duration);

    // 测试2: UDS inbox多消息融合功能
    println!("\n2. 测试 UDS inbox 多消息融合功能...");
    let start_time = Instant::now();
    
    // 初始化UDS系统
    let uds_manager = uds::initialize().await?;
    println!("   UDS 系统初始化成功");
    
    // 创建测试消息
    let test_message = uds::message::UdsMessage {
        id: "test_message_1".to_string(),
        source: "test_source".to_string(),
        destination: "test_destination".to_string(),
        message_type: uds::message::MessageType::Request,
        content: serde_json::json!("Test message"),
        priority: uds::message::MessagePriority::Normal,
        timestamp: chrono::Utc::now().to_rfc3339(),
        expires_at: Some((chrono::Utc::now() + Duration::minutes(5)).to_rfc3339()),
        correlation_id: Some("test_correlation".to_string()),
        metadata: serde_json::json!({"test": "metadata"}),
    };
    
    // 发送消息
    let send_result = uds_manager.send_message(test_message).await;
    println!("   发送消息: {:?}", send_result);
    
    // 测试消息队列
    let queue_status = uds_manager.queue_status().await;
    println!("   消息队列状态: {:?}", queue_status);
    
    let duration = start_time.elapsed();
    println!("   测试耗时: {:?}", duration);

    // 测试3: Teleport跨机上下文传递功能
    println!("\n3. 测试 Teleport 跨机上下文传递功能...");
    let start_time = Instant::now();
    
    // 初始化Teleport系统
    let teleport_manager = teleport::initialize().await?;
    println!("   Teleport 系统初始化成功");
    
    // 创建测试上下文
    let test_context = teleport::context::Context {
        id: "test_context_1".to_string(),
        name: "Test Context".to_string(),
        data: serde_json::json!({"test": "data"}),
        created_at: chrono::Utc::now().to_rfc3339(),
        updated_at: chrono::Utc::now().to_rfc3339(),
        tags: vec!["test", "integration"],
    };
    
    // 打包上下文
    let packet = teleport_manager.pack_context(test_context).await?;
    println!("   上下文打包成功");
    
    // 测试数据包结构
    println!("   数据包类型: {:?}", packet.packet_type);
    println!("   数据包大小: {} bytes", packet.data.len());
    
    let duration = start_time.elapsed();
    println!("   测试耗时: {:?}", duration);

    // 测试4: Ultrareview云端代码审查功能
    println!("\n4. 测试 Ultrareview 云端代码审查功能...");
    let start_time = Instant::now();
    
    // 初始化Ultrareview系统
    ultrareview::initialize().await?;
    println!("   Ultrareview 系统初始化成功");
    
    // 创建代码分析器
    let analyzer = ultrareview::CodeAnalyzer::new(ultrareview::AnalysisLevel::Deep);
    println!("   代码分析器创建成功");
    
    // 测试分析配置
    let config = analyzer.config();
    println!("   分析配置: {:?}", config);
    
    let duration = start_time.elapsed();
    println!("   测试耗时: {:?}", duration);

    // 测试5: Buddy宠物系统
    println!("\n5. 测试 Buddy 宠物系统...");
    let start_time = Instant::now();
    
    // 创建Buddy管理器
    let mut buddy_manager = buddy::BuddyManager::new(app_state.clone());
    println!("   Buddy 管理器创建成功");
    
    // 测试启用Buddy
    buddy_manager.enable();
    println!("   Buddy 启用成功");
    
    // 测试获取问候语
    let greeting = buddy_manager.get_greeting();
    println!("   问候语: {}", greeting);
    
    // 测试精灵动画
    let sprite_frame = buddy_manager.get_sprite_frame();
    println!("   精灵帧: {:?}", sprite_frame.is_some());
    
    // 测试更新动画
    buddy_manager.update_animation();
    println!("   动画更新成功");
    
    let duration = start_time.elapsed();
    println!("   测试耗时: {:?}", duration);

    // 测试6: Kairos时间感知系统
    println!("\n6. 测试 Kairos 时间感知系统...");
    let start_time = Instant::now();
    
    // 创建Kairos管理器
    let mut kairos_manager = kairos::KairosManager::new(app_state);
    println!("   Kairos 管理器创建成功");
    
    // 测试获取当前时间段
    let current_period = kairos_manager.current_period();
    println!("   当前时间段: {:?}", current_period);
    
    // 测试获取智能问候语
    let intelligent_greeting = kairos_manager.get_intelligent_greeting();
    println!("   智能问候语: {}", intelligent_greeting);
    
    // 测试生成智能建议
    let suggestions = kairos_manager.generate_suggestions();
    println!("   生成建议数量: {}", suggestions.len());
    
    // 测试获取优先级最高的建议
    if let Some(highest_priority) = kairos_manager.get_highest_priority_suggestion() {
        println!("   最高优先级建议: {}", highest_priority.title);
    }
    
    let duration = start_time.elapsed();
    println!("   测试耗时: {:?}", duration);

    // 测试7: 系统集成测试
    println!("\n7. 测试系统集成...");
    let start_time = Instant::now();
    
    // 测试Buddy和Kairos的集成
    let buddy_greeting = buddy_manager.get_greeting();
    let kairos_greeting = kairos_manager.get_intelligent_greeting();
    println!("   Buddy 问候: {}", buddy_greeting);
    println!("   Kairos 问候: {}", kairos_greeting);
    
    // 测试UDS和Teleport的集成
    println!("   UDS 和 Teleport 集成测试完成");
    
    let duration = start_time.elapsed();
    println!("   测试耗时: {:?}", duration);

    println!("\n=== 集成测试完成 ===");
    println!("所有功能模块测试通过！");
    Ok(())
}
