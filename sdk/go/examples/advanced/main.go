// Package main demonstrates advanced usage with custom configuration
package main

import (
	"context"
	"fmt"
	"log"
	"net/http"
	"os"
	"time"

	llmcostops "github.com/llm-devops/llm-cost-ops/sdk/go"
	"go.uber.org/zap"
)

// CustomMetrics implements the MetricsCollector interface
type CustomMetrics struct{}

func (m *CustomMetrics) RecordRequest(method string, statusCode int, duration time.Duration) {
	log.Printf("Request: %s, Status: %d, Duration: %v", method, statusCode, duration)
}

func (m *CustomMetrics) RecordError(operation string, err error) {
	log.Printf("Error in %s: %v", operation, err)
}

func main() {
	// Configure custom logger
	logger, err := zap.NewProduction()
	if err != nil {
		log.Fatalf("Failed to create logger: %v", err)
	}
	defer logger.Sync()

	// Configure custom HTTP client with timeout and transport
	httpClient := &http.Client{
		Timeout: 60 * time.Second,
		Transport: &http.Transport{
			MaxIdleConns:        100,
			MaxIdleConnsPerHost: 10,
			IdleConnTimeout:     90 * time.Second,
		},
	}

	// Create client with advanced configuration
	client, err := llmcostops.NewClient(
		llmcostops.WithAPIKey(os.Getenv("COST_OPS_API_KEY")),
		llmcostops.WithBaseURL(os.Getenv("COST_OPS_BASE_URL")),
		llmcostops.WithHTTPClient(httpClient),
		llmcostops.WithLogger(logger),
		llmcostops.WithMaxRetries(5),
		llmcostops.WithRetryDelay(2*time.Second),
		llmcostops.WithRateLimit(50), // 50 requests per second
		llmcostops.WithTimeout(30*time.Second),
		llmcostops.WithMetrics(&CustomMetrics{}),
	)
	if err != nil {
		log.Fatalf("Failed to create client: %v", err)
	}
	defer client.Close()

	// Use context with timeout
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Minute)
	defer cancel()

	// Example 1: Batch ingest with error handling
	if err := batchIngest(ctx, client); err != nil {
		logger.Error("Batch ingest failed", zap.Error(err))
	}

	// Example 2: Cost analytics with trend analysis
	if err := analyzeCosts(ctx, client); err != nil {
		logger.Error("Cost analysis failed", zap.Error(err))
	}

	// Example 3: Schedule automated reports
	if err := scheduleReports(ctx, client); err != nil {
		logger.Error("Report scheduling failed", zap.Error(err))
	}

	// Example 4: Health monitoring
	if err := monitorHealth(ctx, client); err != nil {
		logger.Error("Health check failed", zap.Error(err))
	}

	logger.Info("All operations completed successfully")
}

func batchIngest(ctx context.Context, client *llmcostops.Client) error {
	// Generate multiple usage records
	records := make([]llmcostops.UsageRecord, 0, 100)
	for i := 0; i < 100; i++ {
		records = append(records, llmcostops.UsageRecord{
			ID:        fmt.Sprintf("usage-%d", i),
			Timestamp: time.Now().Add(-time.Duration(i) * time.Hour),
			Provider:  llmcostops.ProviderOpenAI,
			Model: llmcostops.Model{
				Name:          "gpt-4",
				ContextWindow: 8192,
			},
			OrganizationID:   "org-example",
			ProjectID:        "proj-123",
			PromptTokens:     1000 + int64(i*10),
			CompletionTokens: 500 + int64(i*5),
			TotalTokens:      1500 + int64(i*15),
		})
	}

	// Ingest in batches of 10
	batchSize := 10
	for i := 0; i < len(records); i += batchSize {
		end := i + batchSize
		if end > len(records) {
			end = len(records)
		}

		err := client.Usage.Ingest(ctx, &llmcostops.UsageIngestParams{
			Records: records[i:end],
		})
		if err != nil {
			return fmt.Errorf("failed to ingest batch %d: %w", i/batchSize, err)
		}

		log.Printf("Ingested batch %d (%d records)", i/batchSize, end-i)
	}

	return nil
}

func analyzeCosts(ctx context.Context, client *llmcostops.Client) error {
	// Get detailed cost analytics
	analytics, err := client.Costs.Analytics(ctx, &llmcostops.CostAnalyticsParams{
		Range:          llmcostops.RangeLast30Days,
		OrganizationID: "org-example",
		Granularity:    "day",
	})
	if err != nil {
		return fmt.Errorf("failed to get analytics: %w", err)
	}

	fmt.Printf("\nCost Analytics (%s):\n", analytics.Granularity)
	fmt.Printf("Period: %v to %v\n", analytics.Period.Start, analytics.Period.End)
	fmt.Printf("Data points: %d\n", len(analytics.DataPoints))

	if analytics.Trend != nil {
		fmt.Printf("Trend: %s (%.2f%% change, %.2f confidence)\n",
			analytics.Trend.Direction,
			analytics.Trend.ChangeRate*100,
			analytics.Trend.Confidence,
		)
	}

	// Get costs by provider
	byProvider, err := client.Costs.ByProvider(ctx, &llmcostops.CostSummaryParams{
		Range:          llmcostops.RangeLast30Days,
		OrganizationID: "org-example",
	})
	if err != nil {
		return fmt.Errorf("failed to get costs by provider: %w", err)
	}

	fmt.Println("\nCosts by Provider:")
	for provider, cost := range byProvider {
		fmt.Printf("  %s: $%s\n", provider, cost)
	}

	return nil
}

func scheduleReports(ctx context.Context, client *llmcostops.Client) error {
	// Schedule daily cost report
	report, err := client.Export.ScheduleReport(ctx, &llmcostops.ReportScheduleParams{
		Name:           "Daily Cost Report",
		Schedule:       "0 9 * * *", // Daily at 9 AM
		Format:         llmcostops.FormatExcel,
		ReportType:     "cost",
		OrganizationID: "org-example",
		DeliveryMethod: "email",
		DeliveryConfig: map[string]interface{}{
			"to":      []string{"finance@example.com"},
			"subject": "Daily LLM Cost Report",
		},
		Enabled: true,
	})
	if err != nil {
		return fmt.Errorf("failed to schedule report: %w", err)
	}

	fmt.Printf("\nScheduled report: %s (ID: %s)\n", report.Name, report.ID)
	if report.NextRun != nil {
		fmt.Printf("Next run: %s\n", *report.NextRun)
	}

	return nil
}

func monitorHealth(ctx context.Context, client *llmcostops.Client) error {
	// Check overall health
	health, err := client.Health.Check(ctx)
	if err != nil {
		return fmt.Errorf("health check failed: %w", err)
	}

	fmt.Printf("\nHealth Status: %s\n", health.Status)
	fmt.Printf("Version: %s\n", health.Version)
	fmt.Println("\nComponent Health:")
	for component, status := range health.Checks {
		fmt.Printf("  %s: %s", component, status.Status)
		if status.Message != "" {
			fmt.Printf(" (%s)", status.Message)
		}
		fmt.Println()
	}

	return nil
}
