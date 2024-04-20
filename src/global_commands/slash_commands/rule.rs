use crate::sqlx_lib::{get_rule, PostgresPool};
use crate::utils::{embed_response, parse_options};
use crate::{Error, Result};
use serenity::all::{
    ChannelId, CommandInteraction, CommandOptionType, Context, CreateCommand, CreateCommandOption,
    CreateEmbed, Mentionable, ResolvedValue,
};

const CHANNEL_ID: ChannelId = ChannelId::new(747430712617074718);

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
    let guild_id = interaction.guild_id.ok_or_else(|| Error::NoGuild)?;

    let options = interaction.data.options();
    let options = parse_options(&options);

    let rule_id = match options.get("id") {
        Some(ResolvedValue::String(id)) => *id,
        _ => unreachable!("Rule ID is required"),
    };

    let pool = {
        let data = ctx.data.read().await;
        data.get::<PostgresPool>()
            .expect("PostgresPool should exist in data.")
            .clone()
    };

    let rule = get_rule(&pool, rule_id, guild_id.get()).await?;

    embed_response(
        ctx,
        interaction,
        CreateEmbed::new()
            .title(format!("Rule: {}", rule_id))
            .description(format!(
                "**{}.** {}\n\n**Please read the rest of the rules in {}!**",
                rule_id,
                rule,
                CHANNEL_ID.mention()
            )),
    )
    .await?;

    Ok(())
}

pub fn register() -> CreateCommand {
    CreateCommand::new("rule")
        .description("Get a rule")
        .add_option(
            CreateCommandOption::new(CommandOptionType::String, "id", "The ID of the rule")
                .required(true),
        )
}
