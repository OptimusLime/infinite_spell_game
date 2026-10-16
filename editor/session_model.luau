-- The session UI's typed Store boundary. Lists contain full references supplied by Rust.
local model = {}
function model.context() return data.object("runtime.session") end
function model.record(path) return if path and path ~= "" then data.object(path) else nil end
function model.request() return data.object("runtime.session_request") end
function model.select(name) data.object("editor.ui").session = name end
function model.clear_request()
	local request = model.request()
	if request then request.label, request.about, request.id = "", "", "" end
end
return model
