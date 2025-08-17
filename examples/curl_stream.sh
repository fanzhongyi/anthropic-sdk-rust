#!/bin/bash

curl -X POST "${ANTHROPIC_BASE_URL}/v1/messages" \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer ${ANTHROPIC_API_KEY}" \
  -H "anthropic-version: 2023-06-01" \
  -d '{
    "model": "claude-sonnet-4@20250514",
    "max_tokens": 256,
    "stream": true,
    "messages": [
      {
        "role": "user",
        "content": "Please count from 1 to 10 and say hello!"
      }
    ]
  }' --no-buffer

