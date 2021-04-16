const music = require("../../musicFunctions")
const serverConfig = require("../../serverConfigs/745662812335898806.json")

module.exports = {
    commands: ["nowplaying", "np", "song"],
    maxArgs: 0,
    callback: (message, arguments, text) => {
        const queue = music.servers[message.guild.id].queue
        if (queue.nowPlaying) { message.channel.send(`Now Playing: ${queue.nowPlaying.title}`) }
        else { message.reply("Nothing playing. Use `!play` to queue some music up.") }
    },
}