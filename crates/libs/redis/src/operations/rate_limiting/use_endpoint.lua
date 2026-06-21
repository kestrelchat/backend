-- CONFIGURATION_PLACEHOLDER

--- @class RateLimitWindow
--- @field max number
--- @field duration_ms number

--- @class RateLimitBucket
--- @field capacity number
--- @field use_cost number
--- @field duration_ms number

--- @class RateLimitConfig
--- @field short_window RateLimitWindow
--- @field long_window RateLimitWindow
--- @field bucket RateLimitBucket

--- @type RateLimitConfig
--- @diagnostic disable-next-line: undefined-global
local config = config

--- @diagnostic disable-next-line: undefined-global
local KEYS = KEYS or {}

--- @diagnostic disable-next-line: undefined-global
local ARGV = ARGV or {}

--- @class Redis
--- @field call fun(command: string, ...: any): any

--- @type Redis
--- @diagnostic disable-next-line: undefined-global
local redis = redis or {}

local updated_at_key = KEYS[1]
local bucket_key = KEYS[2]
local short_window_key = KEYS[3]
local long_window_key = KEYS[4]

local now_ms = tonumber(ARGV[1])

local updated_at_ms = tonumber(redis.call('GET', updated_at_key))
local bucket = tonumber(redis.call('GET', bucket_key))
local short_window = tonumber(redis.call('GET', short_window_key))
local long_window = tonumber(redis.call('GET', long_window_key))

local retry_after_ms = 0

local short_window_expire = math.ceil(config.short_window.duration_ms - (now_ms % config.short_window.duration_ms))
local long_window_expire = math.ceil(config.long_window.duration_ms - (now_ms % config.long_window.duration_ms))

if not updated_at_ms then
    bucket = config.bucket.capacity
    short_window = 0
    long_window = 0
    updated_at_ms = now_ms
end

local elapsed_ms = math.max(now_ms - updated_at_ms, 0)

if not bucket then
    bucket = config.bucket.capacity
else
    bucket = bucket + (elapsed_ms / config.bucket.duration_ms)
    bucket = math.min(config.bucket.capacity, bucket)
end

if bucket < config.bucket.use_cost then
    retry_after_ms = (config.bucket.use_cost - bucket) * config.bucket.duration_ms
end

if not short_window then
    short_window = 0
else
    local previous_short_window = math.floor(updated_at_ms / config.short_window.duration_ms)
    local current_short_window = math.floor(now_ms / config.short_window.duration_ms)
    if previous_short_window ~= current_short_window then
        short_window = 0
    end
end

if short_window + 1 > config.short_window.max and short_window_expire > retry_after_ms then
    retry_after_ms = short_window_expire
end

if not long_window then
    long_window = 0
else
    local previous_long_window = math.floor(updated_at_ms / config.long_window.duration_ms)
    local current_long_window = math.floor(now_ms / config.long_window.duration_ms)
    if previous_long_window ~= current_long_window then
        long_window = 0
    end
end

if long_window + 1 > config.long_window.max and long_window_expire > retry_after_ms then
    retry_after_ms = long_window_expire
end

-- The cost is deducted regardless of whether the result is success or failure.
--
-- That's the best solution infrastructure-wise, because Redis can put global
-- and per-endpoint rate limiting on different shards, decreasing the coupling.

bucket = math.max(0, bucket - config.bucket.use_cost)
short_window = math.min(config.short_window.max, short_window + 1)
long_window = math.min(config.long_window.max, long_window + 1)
updated_at_ms = now_ms

local bucket_expire = math.ceil((config.bucket.capacity - bucket) * config.bucket.duration_ms)
local updated_at_expire = math.max(bucket_expire, short_window_expire, long_window_expire)

if updated_at_expire > 0 then
    redis.call('SET', updated_at_key, updated_at_ms, 'PX', updated_at_expire)
end
if bucket_expire > 0 then
    redis.call('SET', bucket_key, bucket, 'PX', bucket_expire)
end
if short_window_expire > 0 then
    redis.call('SET', short_window_key, short_window, 'PX', short_window_expire)
end
if long_window_expire > 0 then
    redis.call('SET', long_window_key, long_window, 'PX', long_window_expire)
end

return math.ceil(retry_after_ms * 0.001)
