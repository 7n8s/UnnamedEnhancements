#include "LuauRunner.h"

#include <QByteArray>
#include <QStringList>
#include <lua.h>
#include <lualib.h>
#include <lauxlib.h>

namespace
{
struct OutputContext { QStringList lines; };

int printBridge(lua_State* state)
{
    auto* context = static_cast<OutputContext*>(lua_touserdata(state, lua_upvalueindex(1)));
    QStringList parts;
    const int count = lua_gettop(state);
    for (int i = 1; i <= count; ++i)
    {
        size_t length = 0;
        const char* value = luaL_tolstring(state, i, &length);
        parts.push_back(QString::fromUtf8(value ? value : "", static_cast<qsizetype>(length)));
        lua_pop(state, 1);
    }
    if (context) context->lines.push_back(parts.join('\t'));
    return 0;
}
}

QString LuauRunner::execute(const QString& source)
{
    OutputContext output;
    lua_State* state = luaL_newstate();
    if (!state) return "Runtime error: could not create Luau state.";

    luaL_openlibs(state);
    lua_pushlightuserdata(state, &output);
    lua_pushcclosure(state, printBridge, 1);
    lua_setglobal(state, "print");

    const QByteArray code = source.toUtf8();
    int status = luaL_loadstring(state, code.constData());
    if (status == 0)
        status = lua_pcall(state, 0, LUA_MULTRET, 0);

    if (status != 0)
    {
        size_t length = 0;
        const char* message = lua_tolstring(state, -1, &length);
        output.lines.push_back(QString(status == LUA_ERRSYNTAX ? "Compile error: " : "Runtime error: ")
            + QString::fromUtf8(message ? message : "unknown error", static_cast<qsizetype>(length)));
    }

    lua_close(state);
    return output.lines.isEmpty() ? "Finished with no output." : output.lines.join('\n');
}
