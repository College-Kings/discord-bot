use serenity::all::{
    ButtonStyle, Context, CreateButton, CreateChannel, CreateEmbed, CreateInteractionResponse,
    CreateInteractionResponseMessage, CreateMessage, Mentionable, ModalInteraction,
    PermissionOverwrite, PermissionOverwriteType, Permissions,
};
use zayden_core::parse_modal_data;

use crate::modules::patreon::patreon_member;
use crate::sqlx_lib::PostgresPool;
use crate::{
    guilds::{college_kings::RENDER_REQUESTS_CHANNEL_ID, college_kings_team::MESSY_USER_ID},
    Error, Result,
};

pub async fn run(ctx: &Context, modal: &ModalInteraction) -> Result<()> {
    let guild_id = modal.guild_id.ok_or(Error::MissingGuildId)?;

    let pool = PostgresPool::get(ctx).await;

    let mut data = parse_modal_data(&modal.data.components);

    let response = match data.remove("email") {
        Some(email) => patreon_member(&pool, email, false).await?,
        _ => patreon_member(&pool, &modal.user.id.to_string(), false).await?,
    };

    let tier = response
        .unwrap()
        .data
        .attributes
        .currently_entitled_amount_cents
        / 100;

    let character = data.remove("character").unwrap();
    let prop = data.remove("prop").unwrap_or("No prop specified.");
    let location = data.remove("location").unwrap_or("No location specified.");
    let description = data
        .remove("description")
        .unwrap_or("No description specified.");

    if tier < 50 {
        modal
            .create_response(
                ctx,
                CreateInteractionResponse::Message(
                    CreateInteractionResponseMessage::new()
                        .content("You must be at least a $50 patron to use this feature.\nThe cache is updated every 24 hours. If you've recently upgraded, please wait a day and try again or contact us through the support channel.").ephemeral(true),
                ),
            )
            .await.unwrap();

        return Ok(());
    }

    let request = CreateEmbed::new()
        .title("Render Request")
        .description(description)
        .fields(vec![
            ("Tier", format!("${}", tier).as_str(), true),
            ("Character", character, false),
            ("Prop", prop, false),
            ("Location", location, false),
            ("Description", description, false),
        ]);

    let channel_name: String = format!("{}丨{}", chrono::Utc::now().format("%b"), &modal.user.name)
        .chars()
        .take(100)
        .collect();

    let category_id = RENDER_REQUESTS_CHANNEL_ID
        .to_channel(ctx)
        .await
        .unwrap()
        .guild()
        .unwrap()
        .parent_id
        .unwrap();

    let permissions = vec![
        PermissionOverwrite {
            allow: Permissions::empty(),
            deny: Permissions::VIEW_CHANNEL,
            kind: PermissionOverwriteType::Role(guild_id.everyone_role()),
        },
        PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL | Permissions::MANAGE_CHANNELS,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member(MESSY_USER_ID),
        },
        PermissionOverwrite {
            allow: Permissions::VIEW_CHANNEL,
            deny: Permissions::empty(),
            kind: PermissionOverwriteType::Member(modal.user.id),
        },
    ];

    let channel = guild_id
        .create_channel(
            ctx,
            CreateChannel::new(channel_name)
                .category(category_id)
                .nsfw(true)
                .permissions(permissions),
        )
        .await
        .unwrap();

    channel
        .send_message(
            ctx,
            CreateMessage::new()
                .content(format!(
                    "{} {}",
                    MESSY_USER_ID.mention(),
                    modal.user.mention()
                ))
                .embed(request)
                .button(
                    CreateButton::new("delete_channel")
                        .label("Delete Channel")
                        .style(ButtonStyle::Danger),
                ),
        )
        .await
        .unwrap();

    modal
        .create_response(ctx, CreateInteractionResponse::Acknowledge)
        .await
        .unwrap();

    Ok(())
}
