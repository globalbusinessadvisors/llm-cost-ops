// Package main demonstrates basic usage of the LLM Cost Ops SDK
package main

import (
	"context"
	"fmt"
	"log"
	"time"

	llmcostops "github.com/llm-devops/llm-cost-ops/sdk/go"
)

func main() {
	// Create a new client
	client, err := llmcostops.NewClient(
		llmcostops.WithAPIKey("your-api-key-here"),
		llmcostops.WithBaseURL("https://api.costops.example.com"),
	)
	if err != nil {
		log.Fatalf("Failed to create client: %v", err)
	}
	defer client.Close()

	ctx := context.Background()

	// 1. Add pricing information
	fmt.Println("Adding pricing for GPT-4...")
	pricing, err := client.Pricing.Add(ctx, &llmcostops.PricingAddParams{
		Provider:              llmcostops.ProviderOpenAI,
		Model:                 "gpt-4",
		InputPricePerMillion:  10.0,
		OutputPricePerMillion: 30.0,
		Currency:              llmcostops.CurrencyUSD,
	})
	if err != nil {
		log.Fatalf("Failed to add pricing: %v", err)
	}
	fmt.Printf("Added pricing: %s\n", pricing.ID)

	// 2. Ingest usage data
	fmt.Println("\nIngesting usage data...")
	err = client.Usage.Ingest(ctx, &llmcostops.UsageIngestParams{
		Records: []llmcostops.UsageRecord{
			{
				ID:        "usage-001",
				Timestamp: time.Now(),
				Provider:  llmcostops.ProviderOpenAI,
				Model: llmcostops.Model{
					Name:          "gpt-4",
					ContextWindow: 8192,
				},
				OrganizationID:   "org-example",
				ProjectID:        "proj-123",
				PromptTokens:     1000,
				CompletionTokens: 500,
				TotalTokens:      1500,
			},
		},
	})
	if err != nil {
		log.Fatalf("Failed to ingest usage: %v", err)
	}
	fmt.Println("Usage data ingested successfully")

	// 3. Query usage records
	fmt.Println("\nQuerying usage records...")
	usage, err := client.Usage.List(ctx, &llmcostops.UsageListParams{
		Range:          llmcostops.RangeLast24Hours,
		OrganizationID: "org-example",
	})
	if err != nil {
		log.Fatalf("Failed to query usage: %v", err)
	}
	fmt.Printf("Found %d usage records\n", len(usage))

	// 4. Get cost summary
	fmt.Println("\nGetting cost summary...")
	summary, err := client.Costs.Summary(ctx, &llmcostops.CostSummaryParams{
		Range:          llmcostops.RangeLast30Days,
		OrganizationID: "org-example",
	})
	if err != nil {
		log.Fatalf("Failed to get cost summary: %v", err)
	}
	fmt.Printf("Total cost: %s\n", summary.TotalCost)
	fmt.Printf("Total requests: %d\n", summary.TotalRequests)
	fmt.Printf("Average cost: %s\n", summary.AvgCost)

	// 5. Export data
	fmt.Println("\nExporting data to CSV...")
	data, err := client.Export.Export(ctx, &llmcostops.ExportParams{
		Format:         llmcostops.FormatCSV,
		Range:          llmcostops.RangeLast7Days,
		OrganizationID: "org-example",
		IncludeHeaders: true,
	})
	if err != nil {
		log.Fatalf("Failed to export data: %v", err)
	}
	fmt.Printf("Exported %d bytes\n", len(data))

	fmt.Println("\nDone!")
}
