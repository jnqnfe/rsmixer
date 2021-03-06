mod action_handlers;

use action_handlers::*;

use crate::{
    ui,
    models::{RedrawType, UIMode, RSState},
    Letter, RSError,
};

use tokio::{stream::StreamExt, sync::broadcast::Receiver};

pub async fn event_loop(mut rx: Receiver<Letter>) -> Result<(), RSError> {
    let mut stdout = ui::prepare_terminal()?;

    let mut state = RSState::default();

    ui::draw_page(
        &mut stdout,
        &mut state,
    )
    .await?;

    while let Some(Ok(msg)) = rx.next().await {
        // run action handlers which will decide what to redraw

        if msg == Letter::ExitSignal {
            break;
        }

        state.redraw = general::action_handler(&msg, &mut state).await;

        entries_updates::action_handler(&msg, &mut state).await.apply(&mut state.redraw);

        match state.ui_mode {
            UIMode::Normal => {
                normal::action_handler(&msg, &mut state).await.apply(&mut state.redraw);
            }
            UIMode::ContextMenu => {
                context_menu::action_handler(&msg, &mut state).await.apply(&mut state.redraw);
            }
            UIMode::Help => {
                if msg == Letter::Redraw {
                    state.redraw.take_bigger(RedrawType::Help);
                }
            }
            UIMode::MoveEntry(_, _) => {
               move_entry::action_handler(&msg, &mut state).await.apply(&mut state.redraw);
            }
        };

        scroll::scroll_handler(&msg, &mut state).await?.apply(&mut state.redraw);

        ui::redraw(&mut stdout, &mut state).await?;
    }
    Ok(())
}
