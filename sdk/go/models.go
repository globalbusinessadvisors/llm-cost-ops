package llmcostops

import (
	"time"
)

// Provider represents an LLM provider
type Provider string

const (
	ProviderOpenAI    Provider = "openai"
	ProviderAnthropic Provider = "anthropic"
	ProviderGoogle    Provider = "google"
	ProviderAzure     Provider = "azure"
	ProviderAWS       Provider = "aws"
	ProviderCohere    Provider = "cohere"
	ProviderMistral   Provider = "mistral"
)

// Currency represents a currency code
type Currency string

const (
	CurrencyUSD Currency = "USD"
	CurrencyEUR Currency = "EUR"
	CurrencyGBP Currency = "GBP"
)

// TimeRange represents a predefined time range
type TimeRange string

const (
	RangeLastHour    TimeRange = "last-hour"
	RangeLast24Hours TimeRange = "last-24-hours"
	RangeLast7Days   TimeRange = "last-7-days"
	RangeLast30Days  TimeRange = "last-30-days"
	RangeLast90Days  TimeRange = "last-90-days"
	RangeCustom      TimeRange = "custom"
)

// Model represents an LLM model configuration
type Model struct {
	Name          string `json:"name"`
	Version       string `json:"version,omitempty"`
	ContextWindow int    `json:"context_window,omitempty"`
}

// UsageRecord represents a usage record
type UsageRecord struct {
	ID               string                 `json:"id"`
	Timestamp        time.Time              `json:"timestamp"`
	Provider         Provider               `json:"provider"`
	Model            Model                  `json:"model"`
	OrganizationID   string                 `json:"organization_id"`
	ProjectID        string                 `json:"project_id,omitempty"`
	UserID           string                 `json:"user_id,omitempty"`
	PromptTokens     int64                  `json:"prompt_tokens"`
	CompletionTokens int64                  `json:"completion_tokens"`
	TotalTokens      int64                  `json:"total_tokens"`
	CachedTokens     *int64                 `json:"cached_tokens,omitempty"`
	ReasoningTokens  *int64                 `json:"reasoning_tokens,omitempty"`
	LatencyMs        *int64                 `json:"latency_ms,omitempty"`
	Tags             []string               `json:"tags,omitempty"`
	Metadata         map[string]interface{} `json:"metadata,omitempty"`
	IngestedAt       time.Time              `json:"ingested_at"`
	Source           *Source                `json:"source,omitempty"`
}

// Source represents the source of a usage record
type Source struct {
	Type     string `json:"type"`
	Endpoint string `json:"endpoint,omitempty"`
}

// CostRecord represents a calculated cost record
type CostRecord struct {
	ID         string    `json:"id"`
	UsageID    string    `json:"usage_id"`
	Provider   Provider  `json:"provider"`
	Model      string    `json:"model"`
	InputCost  string    `json:"input_cost"`  // Decimal string
	OutputCost string    `json:"output_cost"` // Decimal string
	TotalCost  string    `json:"total_cost"`  // Decimal string
	Currency   Currency  `json:"currency"`
	Timestamp  time.Time `json:"timestamp"`
}

// PricingTable represents a pricing configuration
type PricingTable struct {
	ID               string           `json:"id"`
	Provider         Provider         `json:"provider"`
	Model            string           `json:"model"`
	PricingStructure PricingStructure `json:"pricing_structure"`
	Currency         Currency         `json:"currency"`
	EffectiveDate    time.Time        `json:"effective_date"`
	EndDate          *time.Time       `json:"end_date,omitempty"`
}

// PricingStructure represents different pricing models
type PricingStructure struct {
	Type string `json:"type"` // "per_token", "per_request", "tiered"

	// For per_token pricing
	InputPricePerMillion  *float64 `json:"input_price_per_million,omitempty"`
	OutputPricePerMillion *float64 `json:"output_price_per_million,omitempty"`
	CachedInputDiscount   *float64 `json:"cached_input_discount,omitempty"`

	// For per_request pricing
	PricePerRequest        *float64 `json:"price_per_request,omitempty"`
	IncludedTokens         *int64   `json:"included_tokens,omitempty"`
	OveragePricePerMillion *float64 `json:"overage_price_per_million,omitempty"`

	// For tiered pricing
	Tiers []PricingTier `json:"tiers,omitempty"`
}

// PricingTier represents a pricing tier
type PricingTier struct {
	Threshold             int64   `json:"threshold"`
	InputPricePerMillion  float64 `json:"input_price_per_million"`
	OutputPricePerMillion float64 `json:"output_price_per_million"`
}

// CostSummary represents aggregated cost data
type CostSummary struct {
	Period        Period              `json:"period"`
	TotalCost     string              `json:"total_cost"`
	TotalRequests int64               `json:"total_requests"`
	AvgCost       string              `json:"avg_cost"`
	ByProvider    map[Provider]string `json:"by_provider"`
	ByModel       map[string]string   `json:"by_model"`
	ByProject     map[string]string   `json:"by_project,omitempty"`
}

// Period represents a time period
type Period struct {
	Start time.Time `json:"start"`
	End   time.Time `json:"end"`
}

// ExportRequest represents an export request
type ExportRequest struct {
	Format         ExportFormat `json:"format"`
	Period         Period       `json:"period"`
	OrganizationID string       `json:"organization_id,omitempty"`
	ProjectID      string       `json:"project_id,omitempty"`
	Filters        *Filters     `json:"filters,omitempty"`
}

// ExportFormat represents export format types
type ExportFormat string

const (
	FormatJSON  ExportFormat = "json"
	FormatCSV   ExportFormat = "csv"
	FormatExcel ExportFormat = "xlsx"
	FormatJSONL ExportFormat = "jsonl"
)

// Filters represents query filters
type Filters struct {
	Providers      []Provider `json:"providers,omitempty"`
	Models         []string   `json:"models,omitempty"`
	OrganizationID string     `json:"organization_id,omitempty"`
	ProjectID      string     `json:"project_id,omitempty"`
	UserID         string     `json:"user_id,omitempty"`
	Tags           []string   `json:"tags,omitempty"`
	MinCost        *float64   `json:"min_cost,omitempty"`
	MaxCost        *float64   `json:"max_cost,omitempty"`
}

// HealthStatus represents the health status of the service
type HealthStatus struct {
	Status    string            `json:"status"`
	Version   string            `json:"version"`
	Timestamp time.Time         `json:"timestamp"`
	Checks    map[string]Health `json:"checks"`
}

// Health represents a health check result
type Health struct {
	Status  string `json:"status"`
	Message string `json:"message,omitempty"`
}

// ListResponse represents a paginated list response
type ListResponse struct {
	Data       interface{} `json:"data"`
	Page       int         `json:"page"`
	PageSize   int         `json:"page_size"`
	TotalCount int64       `json:"total_count"`
	TotalPages int         `json:"total_pages"`
}

// PaginationParams represents pagination parameters
type PaginationParams struct {
	Page     int `json:"page,omitempty"`
	PageSize int `json:"page_size,omitempty"`
}

// SortOrder represents sort order
type SortOrder string

const (
	SortAsc  SortOrder = "asc"
	SortDesc SortOrder = "desc"
)
