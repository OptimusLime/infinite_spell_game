-- Part: the Inspector (Scene, Node and Theme tabs; colors through the picker).
local panels = require("panels")
local picker = require("picker")
return { name = "inspector", slot = "right", order = 1, render = panels.inspector, routes = picker.routes }
