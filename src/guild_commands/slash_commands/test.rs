use serenity::all::{
    ChannelId, CommandInteraction, Context, CreateButton, CreateCommand, CreateMessage, Permissions,
};

use crate::{college_kings::GUILD_ID, utils::message_response, Result};

pub async fn run(ctx: &Context, interaction: &CommandInteraction) -> Result<()> {
    ChannelId::new(1164928484838227968)
        .send_message(
            ctx,
            CreateMessage::default()
                .button(CreateButton::new("production_request").label("Production Request")),
        )
        .await?;

    message_response(ctx, interaction, "Success").await?;
    Ok(())
}

pub async fn register(ctx: &Context) -> Result<()> {
    GUILD_ID
        .create_command(
            ctx,
            CreateCommand::new("test")
                .description("Test command")
                .default_member_permissions(Permissions::ADMINISTRATOR),
        )
        .await?;

    Ok(())
}
