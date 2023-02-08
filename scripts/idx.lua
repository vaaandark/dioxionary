#!/usr/bin/env lua
--[[
Usage: ./idx.lua path/to/idx
       or version 3.0.0
       ./idx.lua -3 path/to/idx
--]]

local f
local pattern
if arg[1] == '-3' then
  pattern=">zI8I8"
  f = arg[2]
else
  pattern=">zI4I4"
  f = arg[1]
end

f = assert(io.open(f, "rb"))
local contents = f:read("a")
local now
local word, offset, size
while not now or now < #contents do
  word, offset, size, now = string.unpack(pattern, contents, now)
  print(string.format("%-50s | %8d | %4d", word, offset, size))
end

f:close()
