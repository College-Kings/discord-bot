use serenity::all::{
    AutoArchiveDuration, ChannelId, ChannelType, Context, CreateEmbed, CreateEmbedFooter,
    CreateInteractionResponse, CreateMessage, CreateThread, InputText, ModalInteraction, Role,
    RoleId,
};

use crate::{
    sqlx_lib::{
        get_support_channel_ids, get_support_role_ids, get_support_thead_id,
        update_support_thread_id, PostgresPool,
    },
    utils::support::{get_thread_name, send_support_message},
    Error, Result,
};

use super::parse_modal_data;

pub async fn run(ctx: &Context, modal: &ModalInteraction) -> Result<()> {
    let guild_id = modal.guild_id.ok_or_else(|| Error::NoGuild)?;

    let pool = {
        let data = ctx.data.read().await;
        data.get::<PostgresPool>()
            .expect("PostgresPool should exist in data.")
            .clone()
    };

    let support_channel_ids = get_support_channel_ids(&pool, guild_id.get()).await?;

    let guild_roles = guild_id.roles(&ctx).await?;

    let support_roles = get_support_role_ids(&pool, guild_id.get())
        .await?
        .into_iter()
        .map(|id| {
            guild_roles
                .get(&RoleId::new(id as u64))
                .ok_or_else(|| Error::NoRole)
        })
        .collect::<Result<Vec<&Role>>>()?;

    let thread_id = get_support_thead_id(&pool, guild_id.get())
        .await
        .unwrap_or(0)
        + 1;

    let data = parse_modal_data(&modal.data.components);
    let content = match data.get("issue") {
        Some(InputText {
            value: Some(value), ..
        }) => value,
        _ => unreachable!("Issue input is required"),
    };

    let thread_name = get_thread_name(thread_id, &modal.user.name, content);

    let version = match data.get("version") {
        Some(InputText {
            value: Some(value), ..
        }) => value,
        _ => unreachable!("Version input is required"),
    };

    let issue = CreateEmbed::default()
        .title("Issue")
        .description(content)
        .footer(CreateEmbedFooter::new(format!("Version: {}", version)));

    let mut messages = vec![CreateMessage::new().embed(issue)];

    if let Some(InputText {
        value: Some(value), ..
    }) = data.get("additional")
    {
        if !value.is_empty() {
            let additional = CreateEmbed::default()
                .title("Additional Information")
                .description(value);
            messages.push(CreateMessage::new().embed(additional));
        }
    }

    for channel_id in support_channel_ids {
        let channel_id = ChannelId::new(channel_id as u64);

        let thread = channel_id
            .create_thread(
                ctx,
                CreateThread::new(&thread_name)
                    .kind(ChannelType::PrivateThread)
                    .auto_archive_duration(AutoArchiveDuration::OneWeek),
            )
            .await?;

        send_support_message(ctx, &thread, &support_roles, &modal.user, messages.clone()).await?;
    }

    update_support_thread_id(&pool, guild_id.get(), thread_id).await?;

    modal
        .create_response(ctx, CreateInteractionResponse::Acknowledge)
        .await?;

    Ok(())
}
