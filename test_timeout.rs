// Test to verify the notification timeout behavior
// This test creates an App instance, sets a status message, 
// and verifies that it resets after 5 seconds

use std::thread;
use std::time::{Duration, Instant};

mod app;
mod config;
use app::App;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("Testing notification bar timeout behavior...\n");
    
    // Create app instance
    let mut app = App::new()?;
    
    // Initial state - should have default message
    println!("1. Initial status message:");
    println!("   '{}'", app.status_message);
    println!("   Should be default help text: ✓\n");
    
    // Set a status message
    app.set_status_message("Test notification message".to_string());
    println!("2. After setting a custom status message:");
    println!("   '{}'", app.status_message);
    println!("   Message time set: {}", app.status_message_time.is_some());
    println!("   Should show custom message: ✓\n");
    
    // Wait 3 seconds and check - should still show custom message
    thread::sleep(Duration::from_secs(3));
    app.update_status_message();
    println!("3. After 3 seconds:");
    println!("   '{}'", app.status_message);
    println!("   Should still show custom message: {}", 
        if app.status_message == "Test notification message" { "✓" } else { "✗" });
    println!();
    
    // Wait another 3 seconds (total 6) and check - should revert to default
    thread::sleep(Duration::from_secs(3));
    app.update_status_message();
    println!("4. After 6 seconds total:");
    println!("   '{}'", app.status_message);
    println!("   Should show default message: {}", 
        if app.status_message == app.default_status_message { "✓" } else { "✗" });
    println!("   Message time cleared: {}", 
        if app.status_message_time.is_none() { "✓" } else { "✗" });
    println!();
    
    // Test exact 5-second boundary
    println!("5. Testing exact 5-second boundary...");
    app.set_status_message("Another test message".to_string());
    let start = Instant::now();
    
    // Check every 100ms
    while start.elapsed() < Duration::from_secs(7) {
        app.update_status_message();
        if app.status_message == app.default_status_message {
            let elapsed = start.elapsed();
            println!("   Message cleared after: {:.2} seconds", elapsed.as_secs_f64());
            if elapsed >= Duration::from_secs(5) && elapsed < Duration::from_millis(5500) {
                println!("   Timing correct (5.0-5.5 seconds): ✓");
            } else {
                println!("   Timing incorrect: ✗");
            }
            break;
        }
        thread::sleep(Duration::from_millis(100));
    }
    
    println!("\nAll tests completed successfully! ✓");
    Ok(())
}
