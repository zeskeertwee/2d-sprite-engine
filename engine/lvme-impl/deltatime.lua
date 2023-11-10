--- Get the delta time since the last frame in milliseconds
---@return number
function deltatime_ms()
    return __deltatime_seconds * 1000.0
end

--- Get the delta time since the last frame in seconds
---@return number
function deltatime()
    return __deltatime_seconds
end