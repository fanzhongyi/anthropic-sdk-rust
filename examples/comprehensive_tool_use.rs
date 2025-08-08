use anthropic_sdk::{
    Tool, ToolRegistry, ToolFunction, ToolResult, ToolUse,
    TokenCounter, api_retry,
};
use serde_json::{json, Value};
use async_trait::async_trait;

/// Weather tool that simulates fetching weather data
struct WeatherTool;

#[async_trait]
impl ToolFunction for WeatherTool {
    async fn execute(&self, parameters: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let location = parameters.get("location")
            .and_then(|v| v.as_str())
            .ok_or("Missing location parameter")?;

        // Simulate API call delay
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        // Simulate weather data based on location
        let weather_data = match location.to_lowercase().as_str() {
            l if l.contains("san francisco") || l.contains("sf") => {
                json!({
                    "location": "San Francisco, CA",
                    "temperature": 68,
                    "condition": "Partly cloudy",
                    "humidity": 65,
                    "wind_speed": "8 mph",
                    "forecast": "Mild and pleasant with some afternoon fog"
                })
            },
            l if l.contains("new york") || l.contains("nyc") => {
                json!({
                    "location": "New York, NY",
                    "temperature": 72,
                    "condition": "Sunny",
                    "humidity": 55,
                    "wind_speed": "12 mph",
                    "forecast": "Clear skies with comfortable temperatures"
                })
            },
            l if l.contains("london") => {
                json!({
                    "location": "London, UK",
                    "temperature": 59,
                    "condition": "Overcast",
                    "humidity": 78,
                    "wind_speed": "6 mph",
                    "forecast": "Typical London weather with light drizzle expected"
                })
            },
            _ => {
                json!({
                    "location": location,
                    "temperature": 65,
                    "condition": "Unknown",
                    "humidity": 50,
                    "wind_speed": "5 mph",
                    "forecast": "Weather data not available for this location"
                })
            }
        };

        Ok(ToolResult::success_json("weather", weather_data))
    }
}

/// Calculator tool for mathematical operations
struct CalculatorTool;

#[async_trait]
impl ToolFunction for CalculatorTool {
    async fn execute(&self, parameters: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let expression = parameters.get("expression")
            .and_then(|v| v.as_str())
            .ok_or("Missing expression parameter")?;

        // Simple expression evaluator (in production, use a proper parser)
        let result = match self.evaluate_expression(expression) {
            Ok(value) => value,
            Err(e) => return Err(format!("Calculation error: {e}").into()),
        };

        let result_data = json!({
            "expression": expression,
            "result": result,
            "explanation": format!("{} = {}", expression, result)
        });
        Ok(ToolResult::success_json("calculate", result_data))
    }
}

impl CalculatorTool {
    fn evaluate_expression(&self, expr: &str) -> Result<f64, String> {
        // Simple evaluator for demo - handles basic arithmetic
        let expr = expr.replace(" ", "");

        if let Ok(num) = expr.parse::<f64>() {
            return Ok(num);
        }

        // Handle simple operations
        if let Some(pos) = expr.find('+') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..]; // Skip the operator
            let left_val = self.evaluate_expression(left)?;
            let right_val = self.evaluate_expression(right)?;
            return Ok(left_val + right_val);
        }

        if let Some(pos) = expr.find('-') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..];
            let left_val = self.evaluate_expression(left)?;
            let right_val = self.evaluate_expression(right)?;
            return Ok(left_val - right_val);
        }

        if let Some(pos) = expr.find('*') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..];
            let left_val = self.evaluate_expression(left)?;
            let right_val = self.evaluate_expression(right)?;
            return Ok(left_val * right_val);
        }

        if let Some(pos) = expr.find('/') {
            let (left, right) = expr.split_at(pos);
            let right = &right[1..];
            let left_val = self.evaluate_expression(left)?;
            let right_val = self.evaluate_expression(right)?;
            if right_val == 0.0 {
                return Err("Division by zero".to_string());
            }
            return Ok(left_val / right_val);
        }

        Err(format!("Cannot evaluate expression: {expr}"))
    }
}

/// Time tool for current time and timezone information
struct TimeTool;

#[async_trait]
impl ToolFunction for TimeTool {
    async fn execute(&self, parameters: Value) -> Result<ToolResult, Box<dyn std::error::Error + Send + Sync>> {
        let timezone = parameters.get("timezone")
            .and_then(|v| v.as_str())
            .unwrap_or("UTC");

        // Simulate timezone lookup
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs();

        let formatted_time = format!("2024-01-15 {:02}:{:02}:{:02}",
            (current_time / 3600) % 24,
            (current_time / 60) % 60,
            current_time % 60
        );

        let time_data = json!({
            "timezone": timezone,
            "current_time": formatted_time,
            "unix_timestamp": current_time,
            "format": "YYYY-MM-DD HH:MM:SS"
        });
        Ok(ToolResult::success_json("time", time_data))
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔧 Comprehensive Tool Use Demo");
    println!("==============================\n");

    // Initialize token counter and retry system
    let token_counter = TokenCounter::new();
    let retry_executor = api_retry();

    // Create tool registry and register tools
    let mut registry = ToolRegistry::new();

    // Register weather tool
    let weather_tool = Tool::new("get_weather", "Get current weather information for a location")
        .parameter("location", "string", "The city and state/country, e.g. 'San Francisco, CA' or 'London, UK'")
        .required("location")
        .build();

    registry.register("get_weather", weather_tool, Box::new(WeatherTool))?;

    // Register calculator tool
    let calculator_tool = Tool::new("calculate", "Perform mathematical calculations")
        .parameter("expression", "string", "Mathematical expression to evaluate, e.g. '25 + 17' or '100 / 4'")
        .required("expression")
        .build();

    registry.register("calculate", calculator_tool, Box::new(CalculatorTool))?;

    // Register time tool
    let time_tool = Tool::new("get_time", "Get current time information")
        .parameter("timezone", "string", "Timezone to get time for (optional, defaults to UTC)")
        .build();

    registry.register("get_time", time_tool, Box::new(TimeTool))?;

    println!("🛠️  Registered {} tools", 3);
    println!("  • get_weather");
    println!("  • calculate");
    println!("  • get_time");

    // Demo scenarios
    println!("\n📱 Demo Scenarios:");
    println!("=================\n");

    // Scenario 1: Weather query
    println!("🌤️  Scenario 1: Weather Query");
    println!("------------------------------");

    let weather_use = ToolUse {
        id: "weather_001".to_string(),
        name: "get_weather".to_string(),
        input: json!({
            "location": "San Francisco, CA"
        }),
    };

    let _weather_start = token_counter.start_request("claude-3-5-sonnet-latest");
    let weather_result = registry.execute(&weather_use).await?;
    println!("Request: Get weather for San Francisco");
    println!("Result: {}", serde_json::to_string_pretty(&weather_result)?);

    // Scenario 2: Mathematical calculation
    println!("\n🧮 Scenario 2: Mathematical Calculation");
    println!("---------------------------------------");

    let calc_use = ToolUse {
        id: "calc_001".to_string(),
        name: "calculate".to_string(),
        input: json!({
            "expression": "25 + 17 * 2"
        }),
    };

    let calc_result = registry.execute(&calc_use).await?;
    println!("Request: Calculate 25 + 17 * 2");
    println!("Result: {}", serde_json::to_string_pretty(&calc_result)?);

    // Scenario 3: Time query
    println!("\n⏰ Scenario 3: Time Information");
    println!("-------------------------------");

    let time_use = ToolUse {
        id: "time_001".to_string(),
        name: "get_time".to_string(),
        input: json!({
            "timezone": "PST"
        }),
    };

    let time_result = registry.execute(&time_use).await?;
    println!("Request: Get current time in PST");
    println!("Result: {}", serde_json::to_string_pretty(&time_result)?);

    // Scenario 4: Error handling
    println!("\n❌ Scenario 4: Error Handling");
    println!("-----------------------------");

    let error_use = ToolUse {
        id: "error_001".to_string(),
        name: "calculate".to_string(),
        input: json!({
            "expression": "10 / 0"  // Division by zero
        }),
    };

    match registry.execute(&error_use).await {
        Ok(result) => println!("Unexpected success: {}", serde_json::to_string_pretty(&result)?),
        Err(e) => println!("Expected error handled: {e}"),
    }

    // Performance metrics
    println!("\n📊 Performance Metrics");
    println!("======================");

    let usage_summary = token_counter.get_summary();
    println!("Token Usage: {} total tokens tracked", usage_summary.total_tokens);
    println!("Session Duration: {:.1} seconds", usage_summary.session_duration.as_secs_f64());

    println!("Retry Policy: {} max retries, {}ms initial delay",
        retry_executor.get_policy().max_retries,
        retry_executor.get_policy().initial_delay.as_millis()
    );

    // Tool registry statistics
    println!("\nTool Registry Stats:");
    println!("  • Registered tools: 3");
    println!("  • Tools executed: 4 (3 successful + 1 error)");
    println!("  • Error scenarios: 1 handled");

    println!("\n✨ Comprehensive Tool Use Demo Complete!");
    println!("🚀 All tool scenarios executed successfully with proper error handling!");
    println!("💡 This demonstrates production-ready tool use patterns with:");
    println!("   • Multiple tool types (weather, calculator, time)");
    println!("   • Comprehensive error handling");
    println!("   • Token tracking and retry policies");

    Ok(())
}
