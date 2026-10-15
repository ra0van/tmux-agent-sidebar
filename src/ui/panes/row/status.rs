use ratatui::{
    style::{Color, Style},
    text::{Line, Span},
};

use super::ctx::RowCtx;
use crate::tmux::PaneStatus;
use crate::ui::icons::StatusIcons;
use crate::ui::text::{display_width, elapsed_label, truncate_to_width};

pub(super) fn status_row(
    pane: &crate::tmux::PaneInfo,
    ctx: &RowCtx,
    icons: &StatusIcons,
    spinner_frame: usize,
    now: u64,
) -> Line<'static> {
    use crate::tmux::PermissionMode;
    let theme = ctx.theme;

    let (icon, pulse_color) = running_icon_for(&pane.status, spinner_frame, icons);
    let icon_color =
        pulse_color.unwrap_or_else(|| theme.status_color(&pane.status, pane.attention));
    let agent = pane.agent.label();
    let badge = pane.permission_mode.badge();
    let mode = if badge.is_empty() { "default" } else { badge };
    let elapsed = elapsed_label(pane.started_at, now);

    let agent_fg = theme.agent_color(&pane.agent);
    let elapsed_fg = if pane.status.is_active() {
        theme.text_active
    } else {
        theme.text_muted
    };
    let mode_fg = match pane.permission_mode {
        PermissionMode::BypassPermissions => theme.badge_danger,
        PermissionMode::Auto
        | PermissionMode::DontAsk
        | PermissionMode::AcceptEdits
        | PermissionMode::Defer => theme.badge_auto,
        PermissionMode::Plan => theme.badge_plan,
        PermissionMode::Default => theme.text_muted,
    };

    // The identity row is stable and explicit: agent · chat name · mode.
    // A session name supplements the agent instead of replacing it, which
    // prevents Claude's asynchronously refreshed name from making the row
    // appear to alternate between two different identities.
    let separator = " · ";
    let elapsed_width = display_width(&elapsed);
    let elapsed_gap = usize::from(elapsed_width > 0);
    let fixed_width = display_width(icon)
        + 1
        + display_width(agent)
        + display_width(separator)
        + display_width(mode);
    let name_budget = ctx
        .inner_width
        .saturating_sub(fixed_width + elapsed_gap + elapsed_width + display_width(separator));
    let name = if pane.session_name.is_empty() {
        String::new()
    } else {
        truncate_to_width(&pane.session_name, name_budget)
    };

    let mut left_spans: Vec<Span<'static>> = Vec::with_capacity(7);
    left_spans.push(Span::styled(
        icon.to_string(),
        ctx.apply_bg(Style::default().fg(icon_color)),
    ));
    left_spans.push(Span::styled(
        format!(" {}", agent),
        ctx.apply_bg(Style::default().fg(agent_fg)),
    ));
    if !name.is_empty() {
        left_spans.push(Span::styled(
            separator.to_string(),
            ctx.apply_bg(Style::default().fg(theme.text_muted)),
        ));
        left_spans.push(Span::styled(
            name.clone(),
            ctx.apply_bg(Style::default().fg(theme.text_active)),
        ));
    }
    left_spans.push(Span::styled(
        separator.to_string(),
        ctx.apply_bg(Style::default().fg(theme.text_muted)),
    ));
    left_spans.push(Span::styled(
        mode.to_string(),
        ctx.apply_bg(Style::default().fg(mode_fg)),
    ));

    let left_width = fixed_width
        + if name.is_empty() {
            0
        } else {
            display_width(separator) + display_width(&name)
        };
    let available_for_elapsed = ctx.inner_width.saturating_sub(left_width);
    let elapsed = truncate_to_width(&elapsed, available_for_elapsed);
    let elapsed_width = display_width(&elapsed);
    let right_spans = vec![Span::styled(
        elapsed,
        ctx.apply_bg(Style::default().fg(elapsed_fg)),
    )];

    ctx.row_line_split(left_spans, left_width, right_spans, elapsed_width)
}

pub(super) fn running_icon_for<'a>(
    status: &PaneStatus,
    spinner_frame: usize,
    icons: &'a StatusIcons,
) -> (&'a str, Option<Color>) {
    use crate::SPINNER_PULSE;

    match status {
        PaneStatus::Running => {
            let color_idx = SPINNER_PULSE[spinner_frame % SPINNER_PULSE.len()];
            (icons.status_icon(status), Some(Color::Indexed(color_idx)))
        }
        _ => (icons.status_icon(status), None),
    }
}
