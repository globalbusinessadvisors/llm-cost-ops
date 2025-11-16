#!/bin/bash
# Health check script for LLM Cost Ops container
# Returns 0 if healthy, 1 if unhealthy

set -e

# Configuration
HEALTH_ENDPOINT="${HEALTH_ENDPOINT:-http://localhost:8080/health}"
TIMEOUT="${HEALTH_CHECK_TIMEOUT:-3}"
MAX_RETRIES="${HEALTH_CHECK_RETRIES:-3}"

# Function to check health endpoint
check_health() {
    local response
    local http_code

    # Make HTTP request with timeout
    response=$(curl -f -s -w "\n%{http_code}" --max-time "$TIMEOUT" "$HEALTH_ENDPOINT" 2>/dev/null || echo "000")
    http_code=$(echo "$response" | tail -n1)

    # Check HTTP status code
    if [ "$http_code" = "200" ]; then
        # Parse JSON response to check status
        local status=$(echo "$response" | head -n-1 | grep -o '"status":"[^"]*"' | cut -d'"' -f4)

        if [ "$status" = "healthy" ] || [ "$status" = "ok" ]; then
            echo "✓ Health check passed (HTTP $http_code, status: $status)"
            return 0
        else
            echo "✗ Health check failed: unhealthy status ($status)"
            return 1
        fi
    else
        echo "✗ Health check failed: HTTP $http_code"
        return 1
    fi
}

# Retry logic
for i in $(seq 1 $MAX_RETRIES); do
    if check_health; then
        exit 0
    fi

    if [ $i -lt $MAX_RETRIES ]; then
        echo "Retrying health check ($i/$MAX_RETRIES)..."
        sleep 1
    fi
done

echo "✗ Health check failed after $MAX_RETRIES retries"
exit 1
