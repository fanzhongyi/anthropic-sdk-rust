#!/bin/bash

curl -X POST "${ANTHROPIC_BASE_URL}/v1/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${ANTHROPIC_API_KEY}" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4@20250514",
    "max_tokens": 2000,
    "stream": true,
    "thinking": {
        "type": "enabled",
        "budget_tokens": 1024
    },
    "messages": [
      {
        "role": "user",
        "content": "Echo hello! and tell me the weather in Beijing and Paris"
      }
    ],
    "tools": [
      {
        "name": "echo",
        "description": "Echo the provided JSON back unchanged",
        "input_schema": {
          "type": "object",
          "properties": {
            "any": {
              "type": "object",
              "description": "Arbitrary JSON to echo back"
            }
          }
        }
      },
      {
        "name": "get_weather",
        "description": "Get current weather for a location",
        "input_schema": {
          "type": "object",
          "properties": {
            "location": {
              "type": "string",
              "description": "City name"
            }
          },
          "required": ["location"]
        }
      }
    ]
  }' --no-buffer

