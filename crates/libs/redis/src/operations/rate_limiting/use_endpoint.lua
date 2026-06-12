-- CONFIGURATION_PLACEHOLDER

---@class RateLimitWindow
---@field max number
---@field duration number

---@class RateLimitBucket
---@field capacity number
---@field use_cost number
---@field fill_interval number
---@field fill_step number

---@class RateLimitConfig
---@field short_window RateLimitWindow
---@field long_window RateLimitWindow
---@field bucket RateLimitBucket

---@type string[]
KEYS = KEYS or {}

---@type string[]
ARGV = ARGV or {}

---@type table
---@diagnostic disable-next-line: lowercase-global
redis = redis or {}

---@type RateLimitConfig
---@diagnostic disable-next-line: undefined-global, lowercase-global
config = config

local UPDATED_AT_KEY = KEYS[1]
local BUCKET_KEY = KEYS[2]
local SHORT_WINDOW_KEY = KEYS[3]
local LONG_WINDOW_KEY = KEYS[4]

-- Return 0 if the request is allowed.
-- Return the required wait time in seconds if the limit is exceeded.
return 0
