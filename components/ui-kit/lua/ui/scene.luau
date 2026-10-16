-- Scene modules own their view and routes; instance state belongs to the app.
-- A scene may include another scene, with a separate key and scoped actions.
local ui = require('ui')
local scene = {}
function scene.action(module, action, args)
    return ui.action('route', {name='scene', args={scene=module, action=action, data=args or {}}})
end
function scene.render(module, app, env, props, key)
    local node = require(module).render(app, env, props or {})
    node.id = key or node.id or module:gsub('[^%w_]', '_')
    return node
end
function scene.enter(app, module, ctx)
    local previous = app.state.scene
    if previous == module then return end
    if previous then
        local old = require(previous)
        if old.exit then old.exit(app, ctx) end
    end
    app.state.scene = module
    local next_scene = require(module)
    if next_scene.enter then next_scene.enter(app, ctx) end
    ctx.set_state(app.state)
end
function scene.dispatch(app, request, ctx)
    -- Scenes explicitly declare the nested scenes allowed to handle actions.
    local function permits(module, target, depth)
        if depth > 16 then return false end
        if module == target then return true end
        for _, child in require(module).children or {} do
            if permits(child, target, depth + 1) then return true end
        end
        return false
    end
    assert(permits(app.state.scene, request.scene, 0), 'scene is not active')
    local routes = require(request.scene).routes or {}
    local action = assert(routes[request.action], 'unknown scene action')
    return action(app, ui.merge(request, request.data), ctx)
end
return scene
