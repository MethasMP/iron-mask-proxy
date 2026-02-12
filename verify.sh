#!/bin/bash

# Colorful UI
GREEN='\033[0;32m'
RED='\033[0;31m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

echo -e "${BLUE}🛡️ Iron Mask Proxy Professional Smoke Test${NC}"
echo -e "${BLUE}===========================================${NC}"

# Check if proxy is running
if ! curl -s http://localhost:3000/healthz > /dev/null; then
    echo -e "${RED}❌ Error: Proxy is not running on http://localhost:3000${NC}"
    echo -e "${YELLOW}Please start it first:${NC}"
    echo -e "${YELLOW}  docker-compose up -d --build${NC}"
    exit 1
fi

run_test() {
    local file=$1
    local desc=$2
    echo -e "\n${YELLOW}🧪 Testing: $desc ($file)${NC}"
    
    # Check if file exists
    if [ ! -f "$file" ]; then
        echo -e "${RED}File not found: $file${NC}"
        return
    fi

    # Show original (raw) input
    echo -e "${BLUE}📤 Input (Raw):${NC}"
    cat "$file" | head -c 200
    echo ""

    # Execute curl - send to proxy's /mask endpoint
    RESPONSE=$(curl -s -X POST http://localhost:3000/mask \
      -H "Content-Type: application/json" \
      -d @"$file")

    if [ $? -eq 0 ] && [ -n "$RESPONSE" ]; then
        echo -e "${GREEN}✅ Success! Response received.${NC}"
        echo -e "${BLUE}📥 Output (Masked):${NC}"
        # Try to format with jq, fall back to raw output
        if command -v jq >/dev/null 2>&1; then
            echo "$RESPONSE" | jq . 2>/dev/null || echo "$RESPONSE" | head -c 500
        else
             echo "$RESPONSE" | head -c 500
        fi
    else
        echo -e "${RED}❌ Failed to send request or empty response.${NC}"
    fi
}

# Run all example tests
run_test "examples/01_simple_log.json" "Basic PII Masking"
run_test "examples/03_mixed_text.json" "Multi-pass Masking (Names/Emails/Phones)"
run_test "examples/05_credit_card_test.json" "Financial Data (Luhn Check)"

echo -e "\n${BLUE}===========================================${NC}"
echo -e "${GREEN}🌟 All smoke tests completed!${NC}"
