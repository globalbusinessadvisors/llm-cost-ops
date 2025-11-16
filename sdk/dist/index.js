'use strict';

Object.defineProperty(exports, '__esModule', { value: true });

/* @llm-cost-ops/sdk - Enterprise-grade TypeScript SDK for LLM Cost Operations */
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __commonJS = (cb, mod) => function __require() {
  return mod || (0, cb[__getOwnPropNames(cb)[0]])((mod = { exports: {} }).exports, mod), mod.exports;
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(
  // If the importer is in node compatibility mode or this is not an ESM
  // file that has been converted to a CommonJS file using a Babel-
  // compatible transform (i.e. "__esModule" has not been set), then set
  // "default" to the CommonJS "module.exports" for node compatibility.
  __defProp(target, "default", { value: mod, enumerable: true }) ,
  mod
));

// node_modules/eventemitter3/index.js
var require_eventemitter3 = __commonJS({
  "node_modules/eventemitter3/index.js"(exports$1, module) {
    var has = Object.prototype.hasOwnProperty;
    var prefix = "~";
    function Events() {
    }
    if (Object.create) {
      Events.prototype = /* @__PURE__ */ Object.create(null);
      if (!new Events().__proto__) prefix = false;
    }
    function EE(fn, context, once) {
      this.fn = fn;
      this.context = context;
      this.once = once || false;
    }
    function addListener(emitter, event, fn, context, once) {
      if (typeof fn !== "function") {
        throw new TypeError("The listener must be a function");
      }
      var listener = new EE(fn, context || emitter, once), evt = prefix ? prefix + event : event;
      if (!emitter._events[evt]) emitter._events[evt] = listener, emitter._eventsCount++;
      else if (!emitter._events[evt].fn) emitter._events[evt].push(listener);
      else emitter._events[evt] = [emitter._events[evt], listener];
      return emitter;
    }
    function clearEvent(emitter, evt) {
      if (--emitter._eventsCount === 0) emitter._events = new Events();
      else delete emitter._events[evt];
    }
    function EventEmitter2() {
      this._events = new Events();
      this._eventsCount = 0;
    }
    EventEmitter2.prototype.eventNames = function eventNames() {
      var names = [], events, name;
      if (this._eventsCount === 0) return names;
      for (name in events = this._events) {
        if (has.call(events, name)) names.push(prefix ? name.slice(1) : name);
      }
      if (Object.getOwnPropertySymbols) {
        return names.concat(Object.getOwnPropertySymbols(events));
      }
      return names;
    };
    EventEmitter2.prototype.listeners = function listeners(event) {
      var evt = prefix ? prefix + event : event, handlers = this._events[evt];
      if (!handlers) return [];
      if (handlers.fn) return [handlers.fn];
      for (var i = 0, l = handlers.length, ee = new Array(l); i < l; i++) {
        ee[i] = handlers[i].fn;
      }
      return ee;
    };
    EventEmitter2.prototype.listenerCount = function listenerCount(event) {
      var evt = prefix ? prefix + event : event, listeners = this._events[evt];
      if (!listeners) return 0;
      if (listeners.fn) return 1;
      return listeners.length;
    };
    EventEmitter2.prototype.emit = function emit(event, a1, a2, a3, a4, a5) {
      var evt = prefix ? prefix + event : event;
      if (!this._events[evt]) return false;
      var listeners = this._events[evt], len = arguments.length, args, i;
      if (listeners.fn) {
        if (listeners.once) this.removeListener(event, listeners.fn, void 0, true);
        switch (len) {
          case 1:
            return listeners.fn.call(listeners.context), true;
          case 2:
            return listeners.fn.call(listeners.context, a1), true;
          case 3:
            return listeners.fn.call(listeners.context, a1, a2), true;
          case 4:
            return listeners.fn.call(listeners.context, a1, a2, a3), true;
          case 5:
            return listeners.fn.call(listeners.context, a1, a2, a3, a4), true;
          case 6:
            return listeners.fn.call(listeners.context, a1, a2, a3, a4, a5), true;
        }
        for (i = 1, args = new Array(len - 1); i < len; i++) {
          args[i - 1] = arguments[i];
        }
        listeners.fn.apply(listeners.context, args);
      } else {
        var length = listeners.length, j;
        for (i = 0; i < length; i++) {
          if (listeners[i].once) this.removeListener(event, listeners[i].fn, void 0, true);
          switch (len) {
            case 1:
              listeners[i].fn.call(listeners[i].context);
              break;
            case 2:
              listeners[i].fn.call(listeners[i].context, a1);
              break;
            case 3:
              listeners[i].fn.call(listeners[i].context, a1, a2);
              break;
            case 4:
              listeners[i].fn.call(listeners[i].context, a1, a2, a3);
              break;
            default:
              if (!args) for (j = 1, args = new Array(len - 1); j < len; j++) {
                args[j - 1] = arguments[j];
              }
              listeners[i].fn.apply(listeners[i].context, args);
          }
        }
      }
      return true;
    };
    EventEmitter2.prototype.on = function on(event, fn, context) {
      return addListener(this, event, fn, context, false);
    };
    EventEmitter2.prototype.once = function once(event, fn, context) {
      return addListener(this, event, fn, context, true);
    };
    EventEmitter2.prototype.removeListener = function removeListener(event, fn, context, once) {
      var evt = prefix ? prefix + event : event;
      if (!this._events[evt]) return this;
      if (!fn) {
        clearEvent(this, evt);
        return this;
      }
      var listeners = this._events[evt];
      if (listeners.fn) {
        if (listeners.fn === fn && (!once || listeners.once) && (!context || listeners.context === context)) {
          clearEvent(this, evt);
        }
      } else {
        for (var i = 0, events = [], length = listeners.length; i < length; i++) {
          if (listeners[i].fn !== fn || once && !listeners[i].once || context && listeners[i].context !== context) {
            events.push(listeners[i]);
          }
        }
        if (events.length) this._events[evt] = events.length === 1 ? events[0] : events;
        else clearEvent(this, evt);
      }
      return this;
    };
    EventEmitter2.prototype.removeAllListeners = function removeAllListeners(event) {
      var evt;
      if (event) {
        evt = prefix ? prefix + event : event;
        if (this._events[evt]) clearEvent(this, evt);
      } else {
        this._events = new Events();
        this._eventsCount = 0;
      }
      return this;
    };
    EventEmitter2.prototype.off = EventEmitter2.prototype.removeListener;
    EventEmitter2.prototype.addListener = EventEmitter2.prototype.on;
    EventEmitter2.prefixed = prefix;
    EventEmitter2.EventEmitter = EventEmitter2;
    if ("undefined" !== typeof module) {
      module.exports = EventEmitter2;
    }
  }
});

// src/errors/index.ts
var CostOpsError = class extends Error {
  name;
  timestamp;
  code;
  constructor(message, code) {
    super(message);
    this.name = this.constructor.name;
    this.code = code;
    this.timestamp = /* @__PURE__ */ new Date();
    if (Error.captureStackTrace !== void 0) {
      Error.captureStackTrace(this, this.constructor);
    }
  }
  /**
   * Convert error to JSON representation
   */
  toJSON() {
    return {
      name: this.name,
      message: this.message,
      code: this.code,
      timestamp: this.timestamp.toISOString(),
      stack: this.stack
    };
  }
};
var ConfigurationError = class extends CostOpsError {
  constructor(message) {
    super(message, "CONFIGURATION_ERROR");
    Object.defineProperty(this, "name", { value: "ConfigurationError" });
  }
};
var ValidationError = class extends CostOpsError {
  field;
  constraints;
  constructor(message, field, constraints) {
    super(message, "VALIDATION_ERROR");
    Object.defineProperty(this, "name", { value: "ValidationError" });
    this.field = field;
    this.constraints = constraints;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      field: this.field,
      constraints: this.constraints
    };
  }
};
var AuthenticationError = class extends CostOpsError {
  constructor(message = "Authentication failed") {
    super(message, "AUTHENTICATION_ERROR");
    Object.defineProperty(this, "name", { value: "AuthenticationError" });
  }
};
var AuthorizationError = class extends CostOpsError {
  requiredPermission;
  constructor(message = "Insufficient permissions", requiredPermission) {
    super(message, "AUTHORIZATION_ERROR");
    Object.defineProperty(this, "name", { value: "AuthorizationError" });
    this.requiredPermission = requiredPermission;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      requiredPermission: this.requiredPermission
    };
  }
};
var ApiError = class extends CostOpsError {
  statusCode;
  response;
  requestId;
  constructor(message, statusCode, response, requestId) {
    super(message, "API_ERROR");
    Object.defineProperty(this, "name", { value: "ApiError" });
    this.statusCode = statusCode;
    this.response = response;
    this.requestId = requestId;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      statusCode: this.statusCode,
      response: this.response,
      requestId: this.requestId
    };
  }
};
var NetworkError = class extends CostOpsError {
  cause;
  constructor(message = "Network request failed", cause) {
    super(message, "NETWORK_ERROR");
    Object.defineProperty(this, "name", { value: "NetworkError" });
    this.cause = cause;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      cause: this.cause?.message
    };
  }
};
var TimeoutError = class extends CostOpsError {
  timeout;
  constructor(message, timeout) {
    super(message, "TIMEOUT_ERROR");
    Object.defineProperty(this, "name", { value: "TimeoutError" });
    this.timeout = timeout;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      timeout: this.timeout
    };
  }
};
var RateLimitError = class extends CostOpsError {
  retryAfter;
  limit;
  constructor(message = "Rate limit exceeded", retryAfter, limit) {
    super(message, "RATE_LIMIT_ERROR");
    Object.defineProperty(this, "name", { value: "RateLimitError" });
    this.retryAfter = retryAfter;
    this.limit = limit;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      retryAfter: this.retryAfter,
      limit: this.limit
    };
  }
};
var NotFoundError = class extends CostOpsError {
  resourceType;
  resourceId;
  constructor(message = "Resource not found", resourceType, resourceId) {
    super(message, "NOT_FOUND_ERROR");
    Object.defineProperty(this, "name", { value: "NotFoundError" });
    this.resourceType = resourceType;
    this.resourceId = resourceId;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      resourceType: this.resourceType,
      resourceId: this.resourceId
    };
  }
};
var ConflictError = class extends CostOpsError {
  conflictingField;
  constructor(message = "Resource conflict", conflictingField) {
    super(message, "CONFLICT_ERROR");
    Object.defineProperty(this, "name", { value: "ConflictError" });
    this.conflictingField = conflictingField;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      conflictingField: this.conflictingField
    };
  }
};
var ServerError = class extends CostOpsError {
  statusCode;
  constructor(message = "Internal server error", statusCode = 500) {
    super(message, "SERVER_ERROR");
    Object.defineProperty(this, "name", { value: "ServerError" });
    this.statusCode = statusCode;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      statusCode: this.statusCode
    };
  }
};
var RetryExhaustedError = class extends CostOpsError {
  attempts;
  lastError;
  constructor(message, attempts, lastError) {
    super(message, "RETRY_EXHAUSTED_ERROR");
    Object.defineProperty(this, "name", { value: "RetryExhaustedError" });
    this.attempts = attempts;
    this.lastError = lastError;
  }
  toJSON() {
    return {
      ...super.toJSON(),
      attempts: this.attempts,
      lastError: this.lastError?.message
    };
  }
};
function isCostOpsError(error) {
  return error instanceof CostOpsError;
}
function isApiError(error) {
  return error instanceof ApiError;
}
function isRetryableError(error) {
  if (error instanceof NetworkError) {
    return true;
  }
  if (error instanceof TimeoutError) {
    return true;
  }
  if (error instanceof RateLimitError) {
    return true;
  }
  if (error instanceof ApiError) {
    return error.statusCode >= 500 || error.statusCode === 429;
  }
  return false;
}

// node_modules/eventemitter3/index.mjs
var import_index = __toESM(require_eventemitter3());
var eventemitter3_default = import_index.default;

// src/middleware/index.ts
var MiddlewareManager = class extends eventemitter3_default {
  requestInterceptors = [];
  responseInterceptors = [];
  errorInterceptors = [];
  /**
   * Add a request interceptor
   */
  addRequestInterceptor(interceptor) {
    this.requestInterceptors.push(interceptor);
    return () => {
      const index = this.requestInterceptors.indexOf(interceptor);
      if (index > -1) {
        this.requestInterceptors.splice(index, 1);
      }
    };
  }
  /**
   * Add a response interceptor
   */
  addResponseInterceptor(interceptor) {
    this.responseInterceptors.push(interceptor);
    return () => {
      const index = this.responseInterceptors.indexOf(interceptor);
      if (index > -1) {
        this.responseInterceptors.splice(index, 1);
      }
    };
  }
  /**
   * Add an error interceptor
   */
  addErrorInterceptor(interceptor) {
    this.errorInterceptors.push(interceptor);
    return () => {
      const index = this.errorInterceptors.indexOf(interceptor);
      if (index > -1) {
        this.errorInterceptors.splice(index, 1);
      }
    };
  }
  /**
   * Process request through all request interceptors
   */
  async processRequest(context) {
    let processedContext = context;
    for (const interceptor of this.requestInterceptors) {
      processedContext = await interceptor(processedContext);
    }
    this.emit("request:start", processedContext);
    return processedContext;
  }
  /**
   * Process response through all response interceptors
   */
  async processResponse(context) {
    let processedContext = context;
    for (const interceptor of this.responseInterceptors) {
      processedContext = await interceptor(processedContext);
    }
    this.emit("request:end", processedContext);
    return processedContext;
  }
  /**
   * Process error through all error interceptors
   */
  async processError(context) {
    let processedContext = context;
    for (const interceptor of this.errorInterceptors) {
      processedContext = await interceptor(processedContext);
    }
    this.emit("request:error", processedContext);
    return processedContext;
  }
  /**
   * Clear all interceptors
   */
  clearInterceptors() {
    this.requestInterceptors = [];
    this.responseInterceptors = [];
    this.errorInterceptors = [];
  }
  /**
   * Get count of registered interceptors
   */
  getInterceptorCount() {
    return {
      request: this.requestInterceptors.length,
      response: this.responseInterceptors.length,
      error: this.errorInterceptors.length
    };
  }
};
function createLoggingInterceptor(debug) {
  return {
    request: (context) => {
      if (debug) {
        console.log("[SDK Request]", {
          method: context.request.method,
          path: context.request.path,
          timestamp: new Date(context.timestamp).toISOString()
        });
      }
      return context;
    },
    response: (context) => {
      if (debug) {
        console.log("[SDK Response]", {
          status: context.response.status,
          duration: context.response.metadata.duration,
          timestamp: new Date(context.timestamp).toISOString()
        });
      }
      return context;
    },
    error: (context) => {
      if (debug) {
        console.error("[SDK Error]", {
          error: context.error.message,
          timestamp: new Date(context.timestamp).toISOString()
        });
      }
      return context;
    }
  };
}
function createAuthInterceptor(apiKey) {
  return (context) => {
    if (!context.request.headers) {
      context.request.headers = {};
    }
    context.request.headers["Authorization"] = `Bearer ${apiKey}`;
    return context;
  };
}
function createMetricsInterceptor() {
  const metrics = {
    totalRequests: 0,
    successfulRequests: 0,
    failedRequests: 0,
    totalDuration: 0,
    averageDuration: 0
  };
  return {
    request: (context) => {
      metrics.totalRequests++;
      return context;
    },
    response: (context) => {
      metrics.successfulRequests++;
      metrics.totalDuration += context.response.metadata.duration;
      metrics.averageDuration = metrics.totalDuration / metrics.successfulRequests;
      return context;
    },
    error: (context) => {
      metrics.failedRequests++;
      return context;
    },
    getMetrics: () => ({ ...metrics })
  };
}

// src/utils/http.ts
async function fetchWithTimeout(url, options = {}, timeout = 3e4) {
  const controller = new AbortController();
  const timeoutId = setTimeout(() => controller.abort(), timeout);
  try {
    const response = await fetch(url, {
      ...options,
      signal: controller.signal
    });
    clearTimeout(timeoutId);
    return response;
  } catch (error) {
    clearTimeout(timeoutId);
    if (error instanceof Error) {
      if (error.name === "AbortError") {
        throw new TimeoutError(`Request timeout after ${timeout}ms`, timeout);
      }
      throw new NetworkError("Network request failed", error);
    }
    throw new NetworkError("Unknown network error");
  }
}
function buildUrl(baseUrl, path, query) {
  const url = new URL(path, baseUrl);
  if (query) {
    Object.entries(query).forEach(([key, value]) => {
      if (value !== void 0 && value !== null) {
        url.searchParams.append(key, String(value));
      }
    });
  }
  return url.toString();
}
async function parseJsonResponse(response) {
  const text = await response.text();
  if (text === "" || text.trim() === "") {
    return {};
  }
  try {
    return JSON.parse(text);
  } catch (error) {
    throw new Error(`Failed to parse JSON response: ${text.substring(0, 100)}`);
  }
}
function extractHeaders(headers) {
  const result = {};
  headers.forEach((value, key) => {
    result[key] = value;
  });
  return result;
}
function isBrowser() {
  return typeof window !== "undefined" && typeof document !== "undefined";
}
function isNode() {
  return typeof process !== "undefined" && process.versions != null && process.versions.node != null;
}
function getUserAgent() {
  const version = "1.0.0";
  if (isNode()) {
    return `llm-cost-ops-sdk-node/${version}`;
  }
  if (isBrowser()) {
    return `llm-cost-ops-sdk-browser/${version}`;
  }
  return `llm-cost-ops-sdk/${version}`;
}

// src/utils/retry.ts
function calculateRetryDelay(attempt, initialDelay, exponentialBackoff, maxDelay = 6e4) {
  if (!exponentialBackoff) {
    return initialDelay;
  }
  const exponentialDelay = initialDelay * Math.pow(2, attempt);
  const jitter = Math.random() * 0.3 * exponentialDelay;
  const delay = Math.min(exponentialDelay + jitter, maxDelay);
  return Math.floor(delay);
}
function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
async function withRetry(fn, options) {
  const {
    maxRetries,
    initialDelay,
    exponentialBackoff,
    maxDelay,
    shouldRetry = isRetryableError,
    onRetry
  } = options;
  let lastError;
  let attempts = 0;
  while (attempts <= maxRetries) {
    try {
      const result = await fn();
      return { result, attempts };
    } catch (error) {
      lastError = error instanceof Error ? error : new Error(String(error));
      attempts++;
      if (attempts > maxRetries || !shouldRetry(error)) {
        break;
      }
      const delay = calculateRetryDelay(attempts - 1, initialDelay, exponentialBackoff, maxDelay);
      if (onRetry) {
        onRetry(attempts, error);
      }
      await sleep(delay);
    }
  }
  throw new RetryExhaustedError(
    `Failed after ${attempts} attempts: ${lastError?.message ?? "Unknown error"}`,
    attempts,
    lastError
  );
}

// src/utils/validation.ts
function validateConfig(config) {
  if (config.baseUrl === void 0 || config.baseUrl === "") {
    throw new ConfigurationError("baseUrl is required");
  }
  try {
    new URL(config.baseUrl);
  } catch {
    throw new ConfigurationError("baseUrl must be a valid URL");
  }
  if (config.timeout !== void 0 && config.timeout <= 0) {
    throw new ConfigurationError("timeout must be a positive number");
  }
  if (config.maxRetries !== void 0 && config.maxRetries < 0) {
    throw new ConfigurationError("maxRetries must be non-negative");
  }
  if (config.retryDelay !== void 0 && config.retryDelay <= 0) {
    throw new ConfigurationError("retryDelay must be a positive number");
  }
}
function validateRequired(value, fieldName) {
  if (value === void 0 || value === null || value === "") {
    throw new ValidationError(`${fieldName} is required`, fieldName);
  }
}
function validateStringLength(value, fieldName, min, max) {
  if (min !== void 0 && value.length < min) {
    throw new ValidationError(
      `${fieldName} must be at least ${min} characters`,
      fieldName,
      { minLength: String(min) }
    );
  }
  if (max !== void 0 && value.length > max) {
    throw new ValidationError(
      `${fieldName} must be at most ${max} characters`,
      fieldName,
      { maxLength: String(max) }
    );
  }
}
function validateNumberRange(value, fieldName, min, max) {
  if (min !== void 0 && value < min) {
    throw new ValidationError(
      `${fieldName} must be at least ${min}`,
      fieldName,
      { min: String(min) }
    );
  }
  if (max !== void 0 && value > max) {
    throw new ValidationError(
      `${fieldName} must be at most ${max}`,
      fieldName,
      { max: String(max) }
    );
  }
}
function validateISODate(value, fieldName) {
  const date = new Date(value);
  if (isNaN(date.getTime())) {
    throw new ValidationError(
      `${fieldName} must be a valid ISO 8601 date string`,
      fieldName
    );
  }
}
function validateNonEmptyArray(value, fieldName) {
  if (!Array.isArray(value) || value.length === 0) {
    throw new ValidationError(`${fieldName} must be a non-empty array`, fieldName);
  }
}
function validateEnum(value, fieldName, validValues) {
  if (!validValues.includes(value)) {
    throw new ValidationError(
      `${fieldName} must be one of: ${validValues.join(", ")}`,
      fieldName,
      { enum: validValues.join(", ") }
    );
  }
}

// src/client/base-client.ts
var BaseClient = class {
  config;
  middleware;
  constructor(config) {
    validateConfig(config);
    this.config = {
      baseUrl: config.baseUrl,
      apiKey: config.apiKey ?? "",
      timeout: config.timeout ?? 3e4,
      maxRetries: config.maxRetries ?? 3,
      retryDelay: config.retryDelay ?? 1e3,
      exponentialBackoff: config.exponentialBackoff ?? true,
      headers: config.headers ?? {},
      debug: config.debug ?? false
    };
    this.middleware = new MiddlewareManager();
    this.setupBuiltInInterceptors();
  }
  /**
   * Setup built-in interceptors
   */
  setupBuiltInInterceptors() {
    if (this.config.apiKey !== "") {
      this.middleware.addRequestInterceptor(createAuthInterceptor(this.config.apiKey));
    }
    if (this.config.debug) {
      const loggingInterceptor = createLoggingInterceptor(this.config.debug);
      this.middleware.addRequestInterceptor(loggingInterceptor.request);
      this.middleware.addResponseInterceptor(loggingInterceptor.response);
      this.middleware.addErrorInterceptor(loggingInterceptor.error);
    }
  }
  /**
   * Make an HTTP request
   */
  async request(options) {
    const startTime = Date.now();
    let requestContext = {
      request: options,
      metadata: {},
      timestamp: startTime
    };
    try {
      requestContext = await this.middleware.processRequest(requestContext);
      const { result, attempts } = await withRetry(
        () => this.executeRequest(requestContext.request),
        {
          maxRetries: options.skipRetry === true ? 0 : this.config.maxRetries,
          initialDelay: this.config.retryDelay,
          exponentialBackoff: this.config.exponentialBackoff,
          onRetry: (attempt, error) => {
            this.middleware.emit("retry:attempt", attempt, error);
          }
        }
      );
      const responseContext = {
        response: {
          ...result,
          metadata: {
            ...result.metadata,
            duration: Date.now() - startTime,
            retries: attempts
          }
        },
        request: requestContext,
        timestamp: Date.now()
      };
      const processedContext = await this.middleware.processResponse(responseContext);
      return processedContext.response;
    } catch (error) {
      const errorContext = {
        error,
        request: requestContext,
        timestamp: Date.now()
      };
      await this.middleware.processError(errorContext);
      throw error;
    }
  }
  /**
   * Execute the actual HTTP request
   */
  async executeRequest(options) {
    const { method, path, query, body, headers = {}, timeout = this.config.timeout } = options;
    const url = buildUrl(this.config.baseUrl, path, query);
    const requestHeaders = {
      "Content-Type": "application/json",
      "User-Agent": getUserAgent(),
      ...this.config.headers,
      ...headers
    };
    const requestOptions = {
      method,
      headers: requestHeaders
    };
    if (body !== void 0 && method !== "GET") {
      requestOptions.body = JSON.stringify(body);
    }
    const response = await fetchWithTimeout(url, requestOptions, timeout);
    const responseHeaders = extractHeaders(response.headers);
    if (!response.ok) {
      await this.handleErrorResponse(response, responseHeaders);
    }
    const data = await parseJsonResponse(response);
    return {
      data,
      status: response.status,
      headers: responseHeaders,
      metadata: {
        requestId: responseHeaders["x-request-id"],
        duration: 0,
        // Will be set by caller
        retries: 0
        // Will be set by caller
      }
    };
  }
  /**
   * Handle error responses
   */
  async handleErrorResponse(response, headers) {
    const status = response.status;
    const requestId = headers["x-request-id"];
    let errorData;
    try {
      errorData = await parseJsonResponse(response);
    } catch {
      errorData = await response.text();
    }
    const errorMessage = this.extractErrorMessage(errorData);
    switch (status) {
      case 401:
        throw new AuthenticationError(errorMessage ?? "Authentication failed");
      case 403:
        throw new AuthorizationError(errorMessage ?? "Insufficient permissions");
      case 404:
        throw new NotFoundError(errorMessage ?? "Resource not found");
      case 409:
        throw new ConflictError(errorMessage ?? "Resource conflict");
      case 429: {
        const retryAfter = headers["retry-after"] !== void 0 ? parseInt(headers["retry-after"], 10) : void 0;
        const rateLimit = headers["x-ratelimit-limit"] !== void 0 ? parseInt(headers["x-ratelimit-limit"], 10) : void 0;
        throw new RateLimitError(errorMessage ?? "Rate limit exceeded", retryAfter, rateLimit);
      }
      case 500:
      case 502:
      case 503:
      case 504:
        throw new ServerError(errorMessage ?? "Internal server error", status);
      default:
        throw new ApiError(errorMessage ?? "API request failed", status, errorData, requestId);
    }
  }
  /**
   * Extract error message from error response
   */
  extractErrorMessage(errorData) {
    if (typeof errorData === "string") {
      return errorData;
    }
    if (typeof errorData === "object" && errorData !== null) {
      const data = errorData;
      return data["message"] ?? data["error"] ?? data["detail"] ?? void 0;
    }
    return void 0;
  }
  /**
   * Make a GET request
   */
  async get(path, query) {
    return this.request({
      method: "GET",
      path,
      query
    });
  }
  /**
   * Make a POST request
   */
  async post(path, body) {
    return this.request({
      method: "POST",
      path,
      body
    });
  }
  /**
   * Make a PUT request
   */
  async put(path, body) {
    return this.request({
      method: "PUT",
      path,
      body
    });
  }
  /**
   * Make a PATCH request
   */
  async patch(path, body) {
    return this.request({
      method: "PATCH",
      path,
      body
    });
  }
  /**
   * Make a DELETE request
   */
  async delete(path) {
    return this.request({
      method: "DELETE",
      path
    });
  }
  /**
   * Get middleware manager for adding custom interceptors
   */
  getMiddleware() {
    return this.middleware;
  }
  /**
   * Get current configuration (read-only)
   */
  getConfig() {
    return { ...this.config };
  }
};

// src/client/cost-ops-client.ts
var CostOpsClient = class extends BaseClient {
  constructor(config) {
    super(config);
  }
  /**
   * Health check endpoint
   */
  async health() {
    const response = await this.get("/health");
    return response.data;
  }
  /**
   * Get cost metrics
   */
  async getMetrics(query) {
    const response = await this.get("/api/v1/metrics", query);
    return response.data;
  }
  /**
   * Create a new cost metric
   */
  async createMetric(metric) {
    validateRequired(metric.service, "service");
    validateRequired(metric.cost, "cost");
    validateRequired(metric.currency, "currency");
    const response = await this.post("/api/v1/metrics", metric);
    return response.data;
  }
  /**
   * Get a specific metric by ID
   */
  async getMetric(id) {
    validateRequired(id, "id");
    const response = await this.get(`/api/v1/metrics/${id}`);
    return response.data;
  }
  /**
   * Delete a metric
   */
  async deleteMetric(id) {
    validateRequired(id, "id");
    await this.delete(`/api/v1/metrics/${id}`);
  }
  /**
   * Get usage statistics
   */
  async getUsageStats(startDate, endDate, services) {
    validateRequired(startDate, "startDate");
    validateRequired(endDate, "endDate");
    const response = await this.get("/api/v1/usage/stats", {
      startDate,
      endDate,
      services: services?.join(",")
    });
    return response.data;
  }
  /**
   * Get all budgets
   */
  async getBudgets() {
    const response = await this.get("/api/v1/budgets");
    return response.data;
  }
  /**
   * Get a specific budget by ID
   */
  async getBudget(id) {
    validateRequired(id, "id");
    const response = await this.get(`/api/v1/budgets/${id}`);
    return response.data;
  }
  /**
   * Create a new budget
   */
  async createBudget(budget) {
    validateRequired(budget.name, "name");
    validateRequired(budget.amount, "amount");
    validateRequired(budget.currency, "currency");
    validateRequired(budget.period, "period");
    validateEnum(budget.period, "period", ["daily", "weekly", "monthly", "yearly"]);
    const response = await this.post("/api/v1/budgets", budget);
    return response.data;
  }
  /**
   * Update a budget
   */
  async updateBudget(id, updates) {
    validateRequired(id, "id");
    const response = await this.patch(`/api/v1/budgets/${id}`, updates);
    return response.data;
  }
  /**
   * Delete a budget
   */
  async deleteBudget(id) {
    validateRequired(id, "id");
    await this.delete(`/api/v1/budgets/${id}`);
  }
  /**
   * Get all alerts
   */
  async getAlerts(acknowledged) {
    const response = await this.get("/api/v1/alerts", {
      acknowledged
    });
    return response.data;
  }
  /**
   * Get a specific alert by ID
   */
  async getAlert(id) {
    validateRequired(id, "id");
    const response = await this.get(`/api/v1/alerts/${id}`);
    return response.data;
  }
  /**
   * Acknowledge an alert
   */
  async acknowledgeAlert(id) {
    validateRequired(id, "id");
    const response = await this.post(`/api/v1/alerts/${id}/acknowledge`);
    return response.data;
  }
  /**
   * Delete an alert
   */
  async deleteAlert(id) {
    validateRequired(id, "id");
    await this.delete(`/api/v1/alerts/${id}`);
  }
  /**
   * Export data
   */
  async exportData(options) {
    validateRequired(options.format, "format");
    validateRequired(options.dateRange, "dateRange");
    validateEnum(options.format, "format", ["json", "csv", "xlsx", "pdf"]);
    const response = await this.post("/api/v1/export", options);
    return response.data;
  }
  /**
   * Get cost forecast
   */
  async getForecast(period, services) {
    validateRequired(period, "period");
    validateEnum(period, "period", ["week", "month", "quarter", "year"]);
    const response = await this.get("/api/v1/forecast", {
      period,
      services: services?.join(",")
    });
    return response.data;
  }
  /**
   * Get all webhooks
   */
  async getWebhooks() {
    const response = await this.get("/api/v1/webhooks");
    return response.data;
  }
  /**
   * Get a specific webhook by ID
   */
  async getWebhook(id) {
    validateRequired(id, "id");
    const response = await this.get(`/api/v1/webhooks/${id}`);
    return response.data;
  }
  /**
   * Create a new webhook
   */
  async createWebhook(webhook) {
    validateRequired(webhook.url, "url");
    validateRequired(webhook.events, "events");
    const response = await this.post("/api/v1/webhooks", webhook);
    return response.data;
  }
  /**
   * Update a webhook
   */
  async updateWebhook(id, updates) {
    validateRequired(id, "id");
    const response = await this.patch(`/api/v1/webhooks/${id}`, updates);
    return response.data;
  }
  /**
   * Delete a webhook
   */
  async deleteWebhook(id) {
    validateRequired(id, "id");
    await this.delete(`/api/v1/webhooks/${id}`);
  }
  /**
   * Test a webhook
   */
  async testWebhook(id) {
    validateRequired(id, "id");
    const response = await this.post(
      `/api/v1/webhooks/${id}/test`
    );
    return response.data;
  }
};

exports.ApiError = ApiError;
exports.AuthenticationError = AuthenticationError;
exports.AuthorizationError = AuthorizationError;
exports.BaseClient = BaseClient;
exports.ConfigurationError = ConfigurationError;
exports.ConflictError = ConflictError;
exports.CostOpsClient = CostOpsClient;
exports.CostOpsError = CostOpsError;
exports.MiddlewareManager = MiddlewareManager;
exports.NetworkError = NetworkError;
exports.NotFoundError = NotFoundError;
exports.RateLimitError = RateLimitError;
exports.RetryExhaustedError = RetryExhaustedError;
exports.ServerError = ServerError;
exports.TimeoutError = TimeoutError;
exports.ValidationError = ValidationError;
exports.buildUrl = buildUrl;
exports.calculateRetryDelay = calculateRetryDelay;
exports.createAuthInterceptor = createAuthInterceptor;
exports.createLoggingInterceptor = createLoggingInterceptor;
exports.createMetricsInterceptor = createMetricsInterceptor;
exports.default = CostOpsClient;
exports.extractHeaders = extractHeaders;
exports.fetchWithTimeout = fetchWithTimeout;
exports.getUserAgent = getUserAgent;
exports.isApiError = isApiError;
exports.isBrowser = isBrowser;
exports.isCostOpsError = isCostOpsError;
exports.isNode = isNode;
exports.isRetryableError = isRetryableError;
exports.parseJsonResponse = parseJsonResponse;
exports.sleep = sleep;
exports.validateConfig = validateConfig;
exports.validateEnum = validateEnum;
exports.validateISODate = validateISODate;
exports.validateNonEmptyArray = validateNonEmptyArray;
exports.validateNumberRange = validateNumberRange;
exports.validateRequired = validateRequired;
exports.validateStringLength = validateStringLength;
exports.withRetry = withRetry;
//# sourceMappingURL=index.js.map
//# sourceMappingURL=index.js.map