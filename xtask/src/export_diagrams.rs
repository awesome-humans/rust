//! `export-diagrams`: draws this repository's diagrams as SVG into `assets/diagrams/`: the
//! maintenance pipeline, which shows each skill from what starts it to where its result lands,
//! source standing, which shows how vet-source sorts a candidate, and the research lifecycle.
//! The output is committed, so rerun this after changing a diagram here, and never edit the
//! SVG by hand. CI reruns it and fails when the committed SVGs differ from what it draws.
//!
//! Each diagram is data at the top of its function, rows or nodes with their labels, and
//! layout code below. Colours and fonts come from the Awesome Humans design system, so these
//! match the diagrams on awesome-humans.net. Text names IBM Plex where the reader has it
//! installed and falls back to system fonts, so a diagram makes no third-party requests.
//! Colours follow the reader's light or dark preference through prefers-color-scheme. Every
//! diagram paints its own background, because GitHub can show a README in a theme other than
//! the operating system's, and a transparent diagram would then put light text on a light page.

use std::fmt::Write as _;
use std::fs;
use std::io;

use crate::repo::Repo;

pub const OUTPUT: &str = "assets/diagrams";

pub fn run(repo: &Repo) -> io::Result<()> {
    let directory = repo.path(OUTPUT);
    fs::create_dir_all(&directory)?;
    fs::write(directory.join("maintenance-pipeline.svg"), pipeline())?;
    fs::write(directory.join("source-standing.svg"), standing())?;
    fs::write(directory.join("research-lifecycle.svg"), research())?;
    Ok(())
}

// One skill: what starts it, the branch it works on, what the owner does at the human gate and
// where its result lands, which is "roster", "research", "opinions" or "report". A report-only
// skill has no branch and opens no pull request, so it passes neither gate.
struct Row {
    lane: &'static str,
    picks_up: &'static [&'static str],
    skill: &'static str,
    branch: Option<&'static str>,
    human_gate: &'static str,
    lands_in: &'static str,
}

const ROWS: &[Row] = &[
    Row {
        lane: "sources",
        picks_up: &["candidate source,", "or a change in standing"],
        skill: "vet-source",
        branch: Some("vet/<source-id>"),
        human_gate: "check evidence",
        lands_in: "roster",
    },
    Row {
        lane: "research",
        picks_up: &["a question", "about <topic>"],
        skill: "research-topic",
        branch: Some("research/<topic>"),
        human_gate: "review",
        lands_in: "research",
    },
    Row {
        lane: "research",
        picks_up: &["topic saved in", "research/"],
        skill: "resolve-research",
        branch: Some("resolve/<topic>"),
        human_gate: "approve weave",
        lands_in: "opinions",
    },
    Row {
        lane: "opinions",
        picks_up: &["monthly harvest", "reminder issue"],
        skill: "harvest-sources",
        branch: Some("harvest/<date>"),
        human_gate: "review",
        lands_in: "opinions",
    },
    Row {
        lane: "opinions",
        picks_up: &["release-watch PR,", "or a newer pin"],
        skill: "refresh-rust-versions",
        branch: Some("refresh/rust-<version>"),
        human_gate: "review",
        lands_in: "opinions",
    },
    Row {
        lane: "opinions",
        picks_up: &["HOUSE-OPINIONS.md", "inbox entry"],
        skill: "weave-house-opinion",
        branch: Some("house/<slug>"),
        human_gate: "confirm wording",
        lands_in: "opinions",
    },
    Row {
        lane: "reports",
        picks_up: &["periodic cadence"],
        skill: "audit-freshness",
        branch: None,
        human_gate: "",
        lands_in: "report",
    },
    Row {
        lane: "reports",
        picks_up: &["periodic cadence,", "or before an edition"],
        skill: "review-opinions",
        branch: None,
        human_gate: "",
        lands_in: "report",
    },
    Row {
        lane: "reports",
        picks_up: &["a project path", "or repository URL"],
        skill: "review-project",
        branch: None,
        human_gate: "",
        lands_in: "report",
    },
];

const LANES: &[(&str, &str)] = &[
    ("sources", "Sources"),
    ("research", "Research"),
    ("opinions", "Opinions"),
    ("reports", "Reports"),
];

fn rows_where(predicate: impl Fn(&Row) -> bool) -> Vec<usize> {
    (0..ROWS.len()).filter(|&i| predicate(&ROWS[i])).collect()
}

#[expect(
    clippy::too_many_lines,
    reason = "a straight port of the .NET script's one function, kept in its order so the two can be compared"
)]
fn pipeline() -> String {
    const TITLE: &str = "Maintenance pipeline";
    const DESCRIPTION: &str = "Swimlane diagram of how this repository maintains itself: vet-source, research-topic, resolve-research, harvest-sources, refresh-rust-versions, \
        and weave-house-opinion each work on their own branch, pass the CI checks, get an agent review, pass the owner's review, and land in the roster, research, or the opinions and templates; \
        audit-freshness, review-opinions, and review-project only report. Their findings, the release watch, and the harvest reminder start the next run.";

    // Columns, left to right, and the height of one row.
    const X_BAND: f64 = 12.0;
    const W_BAND: f64 = 96.0;
    const X_BRACKET: f64 = 118.0;
    const X_PICKS_UP: f64 = 128.0;
    const W_PICKS_UP: f64 = 192.0;
    const X_SKILL: f64 = 344.0;
    const W_SKILL: f64 = 236.0;
    const X_AGENT_GATE: f64 = 604.0;
    const W_AGENT_GATE: f64 = 116.0;
    const GATE_HALF: f64 = 20.0;
    const X_HUMAN_GATE: f64 = 764.0;
    const X_LANDS_IN: f64 = 910.0;
    const W_LANDS_IN: f64 = 184.0;
    const WIDTH: f64 = 1106.0;
    const ROW_HEIGHT: f64 = 66.0;

    // The box that the loop of findings leads back to.
    const START_X: f64 = 460.0;
    const START_WIDTH: f64 = 402.0;

    // Rows stack with a gap between lanes.
    let mut ys = Vec::new();
    let mut y = 44.0;
    let mut previous: Option<&str> = None;
    for row in ROWS {
        if previous.is_some_and(|lane| lane != row.lane) {
            y += 18.0;
        }

        ys.push(y);
        y += ROW_HEIGHT + 8.0;
        previous = Some(row.lane);
    }

    let rows_bottom = ys[ys.len() - 1] + ROW_HEIGHT;
    let mut s = String::new();

    for (x, label) in [
        (X_BAND, "lane"),
        (X_PICKS_UP, "picks up"),
        (X_SKILL, "skill"),
        (X_AGENT_GATE, "agent gate"),
        (X_HUMAN_GATE - GATE_HALF, "human gate"),
        (X_LANDS_IN, "lands in"),
    ] {
        s += &text(x, 24.0, label, "th", "start");
    }

    for (key, name) in LANES {
        let rows = rows_where(|row| row.lane == *key);
        let top = ys[rows[0]];
        let bottom = ys[rows[rows.len() - 1]] + ROW_HEIGHT;
        let _ = write!(
            s,
            "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"6\" fill=\"var(--{key})\" opacity=\".14\"/>",
            n(X_BAND),
            n(top),
            n(W_BAND),
            n(bottom - top)
        );
        s += &accent_edge(X_BAND, top, bottom - top, &format!("var(--{key})"));
        s += &text(
            X_BAND + 14.0,
            f64::midpoint(top, bottom) + 6.0,
            name,
            "tb",
            "start",
        );
    }

    for (row, &y0) in ROWS.iter().zip(&ys) {
        let cy = y0 + ROW_HEIGHT / 2.0;

        s += &rect(
            X_PICKS_UP,
            y0 + 6.0,
            W_PICKS_UP,
            ROW_HEIGHT - 12.0,
            6.0,
            "chip",
        );
        s += &lines(X_PICKS_UP + 12.0, cy, row.picks_up);
        s += &arrow(X_PICKS_UP + W_PICKS_UP, cy, X_SKILL - 2.0, cy, false);

        s += &rect(X_SKILL, y0 + 4.0, W_SKILL, ROW_HEIGHT - 8.0, 6.0, "box");
        s += &accent_edge(
            X_SKILL,
            y0 + 4.0,
            ROW_HEIGHT - 8.0,
            &format!("var(--{})", row.lane),
        );
        s += &text(X_SKILL + 14.0, cy - 4.0, row.skill, "tm", "start");
        s += &text(
            X_SKILL + 14.0,
            cy + 16.0,
            row.branch.unwrap_or("report only"),
            "ts",
            "start",
        );

        if row.branch.is_some() {
            s += &arrow(X_SKILL + W_SKILL, cy, X_AGENT_GATE - 2.0, cy, false);
            s += &rect(
                X_AGENT_GATE,
                cy - GATE_HALF,
                W_AGENT_GATE,
                GATE_HALF * 2.0,
                6.0,
                "check",
            );
            s += &text(
                X_AGENT_GATE + W_AGENT_GATE / 2.0,
                cy - 3.0,
                "CI checks,",
                "check-t",
                "middle",
            );
            s += &text(
                X_AGENT_GATE + W_AGENT_GATE / 2.0,
                cy + 13.0,
                "agent review",
                "check-t",
                "middle",
            );
            s += &arrow(
                X_AGENT_GATE + W_AGENT_GATE,
                cy,
                X_HUMAN_GATE - GATE_HALF - 2.0,
                cy,
                false,
            );

            // The human gate is a diamond the height of the agent gate's box.
            s += &diamond(X_HUMAN_GATE, cy, GATE_HALF, "gate");
            s += &arrow(X_HUMAN_GATE + GATE_HALF, cy, X_LANDS_IN - 2.0, cy, false);
            s += &text(
                X_HUMAN_GATE + GATE_HALF + 8.0,
                cy - 9.0,
                row.human_gate,
                "gate-t",
                "start",
            );
        } else {
            s += &arrow(X_SKILL + W_SKILL, cy, X_LANDS_IN - 2.0, cy, true);
            s += &text(
                f64::midpoint(X_AGENT_GATE, X_LANDS_IN),
                cy - 8.0,
                "no pull request",
                "ts",
                "middle",
            );
        }
    }

    // Each destination is one store spanning the rows that land in it.
    for (key, class, labels, subtitle) in [
        ("roster", "store", &["AWESOME-HUMANS.md"][..], "the roster"),
        (
            "research",
            "store",
            &["research/"][..],
            "staged, unresolved",
        ),
        (
            "opinions",
            "store",
            &["opinions/", "templates/"][..],
            "distilled guidance",
        ),
        ("report", "ext", &["a report"][..], "never saved here"),
    ] {
        let rows = rows_where(|row| row.lands_in == key);
        let top = ys[rows[0]] + 4.0;
        let bottom = ys[rows[rows.len() - 1]] + ROW_HEIGHT - 4.0;
        s += &rect(X_LANDS_IN, top, W_LANDS_IN, bottom - top, 6.0, class);
        let count = f64::from(u32::try_from(labels.len()).unwrap_or(1));
        let ty = f64::midpoint(top, bottom) - count * 11.0 + 5.0;
        let label_class = if key == "report" { "tb" } else { "tm" };
        let mut offset = 0.0;
        for label in labels {
            s += &text(X_LANDS_IN + 14.0, ty + offset, label, label_class, "start");
            offset += 22.0;
        }

        s += &text(X_LANDS_IN + 14.0, ty + offset, subtitle, "ts", "start");
    }

    // The loop: a report's findings, with the scheduled workflows and the owner, start the
    // next run in whichever row it belongs to.
    let loop_y = rows_bottom + 56.0;
    s += &path_arrow(
        &format!(
            "M{} {}V{}H{}",
            n(X_LANDS_IN + W_LANDS_IN / 2.0),
            n(rows_bottom - 4.0),
            n(loop_y),
            n(START_X + START_WIDTH + 2.0)
        ),
        false,
    );
    s += &text(
        X_LANDS_IN + W_LANDS_IN / 2.0 - 8.0,
        loop_y - 8.0,
        "findings",
        "ts",
        "end",
    );
    s += &rect(START_X, loop_y - 34.0, START_WIDTH, 68.0, 6.0, "box");
    s += &text(
        START_X + 16.0,
        loop_y - 5.0,
        "What starts a run",
        "tb",
        "start",
    );
    s += &text(
        START_X + 16.0,
        loop_y + 17.0,
        "findings, the release watch, the harvest reminder, you",
        "ts",
        "start",
    );
    let _ = write!(
        s,
        "<path d=\"M{} {}H{}V{}\" class=\"edge\"/>",
        n(START_X),
        n(loop_y),
        n(X_BRACKET),
        n(ys[0] + ROW_HEIGHT / 2.0)
    );
    s += &text(
        f64::midpoint(START_X, X_BRACKET),
        loop_y - 8.0,
        "starts the next skill",
        "ts",
        "middle",
    );
    for y0 in &ys {
        s += &arrow(
            X_BRACKET,
            y0 + ROW_HEIGHT / 2.0,
            X_PICKS_UP - 1.0,
            y0 + ROW_HEIGHT / 2.0,
            false,
        );
    }

    document(WIDTH, loop_y + 50.0, TITLE, DESCRIPTION, &s)
}

#[expect(
    clippy::too_many_lines,
    reason = "a straight port of the .NET script's one function, kept in its order so the two can be compared"
)]
fn standing() -> String {
    const TITLE: &str = "Source standing";
    const DESCRIPTION: &str = "Flow diagram: vet-source sorts a candidate source into citable, with two years of writing and all four criteria; the watch list, under two years or dormant; \
        or declined. A citable source is unmarked, marked Corroborate and never the only citation on a claim, or marked Discovery-only and never cited. \
        Citable and watch-listed sources go back through vet-source when they go dormant, drop in quality, or clear a blocker.";

    // Each marking a citable source can carry, and what it lets a citation rest on.
    const MARKINGS: &[(&str, &str)] = &[
        ("unmarked", "may carry a claim alone"),
        (
            "**Corroborate.**",
            "cited, never as the only source on a claim",
        ),
        (
            "**Discovery-only.**",
            "never cited, only followed to its primary source",
        ),
    ];

    // The candidate sits above vet-source, and each marking's rule sits under it, so the
    // diagram is narrow enough for its text to stay legible in a README. Boxes leave room for
    // DejaVu Sans, the widest fallback a reader without IBM Plex sees.
    const CANDIDATE_X: f64 = 20.0;
    const CANDIDATE_Y: f64 = 20.0;
    const CANDIDATE_WIDTH: f64 = 232.0;
    const VET_X: f64 = 136.0;
    const VET_Y: f64 = 196.0;
    const VET_HALF: f64 = 38.0;
    const SPINE_X: f64 = 276.0;
    const OUT_X: f64 = 508.0;
    const OUT_WIDTH: f64 = 416.0;
    const LOOP_X: f64 = 1066.0;
    const CHIP_HEIGHT: f64 = 52.0;
    const CITABLE_Y: f64 = 20.0;

    let mut s = String::new();
    s += &node(
        CANDIDATE_X,
        CANDIDATE_Y,
        CANDIDATE_WIDTH,
        64.0,
        ("Candidate source", "blog, docs, book, newsletter"),
        "box",
        "tb",
        None,
    );
    s += &arrow(
        VET_X,
        CANDIDATE_Y + 64.0,
        VET_X,
        VET_Y - VET_HALF - 2.0,
        false,
    );
    s += &diamond(VET_X, VET_Y, VET_HALF, "decision");
    s += &text(
        VET_X + 12.0,
        CANDIDATE_Y + 64.0 + 40.0,
        "vet-source",
        "tm",
        "start",
    );
    s += &text(VET_X, VET_Y + 4.0, "criteria", "ts", "middle");
    let _ = write!(
        s,
        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"edge\"/>",
        n(VET_X + VET_HALF),
        n(VET_Y),
        n(SPINE_X),
        n(VET_Y)
    );

    // Citable, with its markings inside.
    let citable_height = 64.0 + 3.0 * (CHIP_HEIGHT + 8.0) + 8.0;
    s += &rect(OUT_X, CITABLE_Y, OUT_WIDTH, citable_height, 8.0, "box");
    s += &accent_edge(OUT_X, CITABLE_Y, citable_height, "var(--accent)");
    s += &text(OUT_X + 16.0, CITABLE_Y + 27.0, "Citable", "tb", "start");
    s += &text(
        OUT_X + 16.0,
        CITABLE_Y + 48.0,
        "quotable in opinions and templates, within its marking",
        "ts",
        "start",
    );
    let mut y0 = CITABLE_Y + 64.0;
    for (marking, rule) in MARKINGS {
        s += &rect(OUT_X + 16.0, y0, OUT_WIDTH - 32.0, CHIP_HEIGHT, 6.0, "chip");
        s += &text(OUT_X + 28.0, y0 + 22.0, marking, "tm", "start");
        s += &text(OUT_X + 28.0, y0 + 42.0, rule, "ts", "start");
        y0 += CHIP_HEIGHT + 8.0;
    }

    let watch_y = CITABLE_Y + citable_height + 24.0;
    let declined_y = watch_y + 84.0;
    let loop_y = declined_y + 60.0 + 36.0;
    s += &node(
        OUT_X,
        watch_y,
        OUT_WIDTH,
        60.0,
        ("Watch list", "leads only, never cited"),
        "ext",
        "tb",
        None,
    );
    s += &node(
        OUT_X,
        declined_y,
        OUT_WIDTH,
        60.0,
        ("Declined", "off the roster, the decision logged"),
        "ext",
        "tb",
        None,
    );

    for (y, label) in [
        (CITABLE_Y + 32.0, "two years and all four criteria"),
        (watch_y + 30.0, "under two years, or dormant"),
        (declined_y + 30.0, "otherwise"),
    ] {
        s += &path_arrow(
            &format!("M{} {}V{}H{}", n(SPINE_X), n(VET_Y), n(y), n(OUT_X - 2.0)),
            false,
        );
        s += &text(SPINE_X + 12.0, y - 8.0, label, "ts", "start");
    }

    // Standing is never permanent: citable and watch-listed sources share one path back.
    let _ = write!(
        s,
        "<path d=\"M{} {}H{}V{}\" class=\"edge dash\"/>",
        n(OUT_X + OUT_WIDTH),
        n(CITABLE_Y + 32.0),
        n(LOOP_X),
        n(loop_y)
    );
    let _ = write!(
        s,
        "<path d=\"M{} {}H{}\" class=\"edge dash\"/>",
        n(OUT_X + OUT_WIDTH),
        n(watch_y + 30.0),
        n(LOOP_X)
    );
    s += &path_arrow(
        &format!(
            "M{} {}H{}V{}",
            n(LOOP_X),
            n(loop_y),
            n(VET_X),
            n(VET_Y + VET_HALF + 2.0)
        ),
        true,
    );
    s += &text(
        OUT_X + OUT_WIDTH + 10.0,
        CITABLE_Y + 24.0,
        "dormant, weaker",
        "ts",
        "start",
    );
    s += &text(
        OUT_X + OUT_WIDTH + 10.0,
        watch_y + 22.0,
        "blocker clears",
        "ts",
        "start",
    );
    s += &text(
        f64::midpoint(VET_X, LOOP_X),
        loop_y - 8.0,
        "re-vetted: standing is never permanent",
        "ts",
        "middle",
    );

    document(LOOP_X + 20.0, loop_y + 28.0, TITLE, DESCRIPTION, &s)
}

#[expect(
    clippy::too_many_lines,
    reason = "a straight port of the .NET script's one function, kept in its order so the two can be compared"
)]
fn research() -> String {
    const TITLE: &str = "Research lifecycle";
    const DESCRIPTION: &str = "Flow diagram: research-topic opens a pull request that saves research/<topic>.md; resolve-research then promotes it into the opinions and templates, \
        discards it, or promotes part of it, deleting the topic in the same pull request. A blocked remainder stays staged and goes back through resolve-research \
        once vet-source clears the source it waits on.";

    const OUTCOMES: &[(&str, &str)] = &[
        ("Promote", "opinions and templates updated, topic deleted"),
        ("Discard", "nothing woven, topic deleted"),
        (
            "Partial promotion",
            "the rest woven, the blocked remainder stays staged",
        ),
    ];

    // research-topic sits above the topic it saves, so the diagram is narrow enough for its
    // text to stay legible in a README.
    const CENTER_Y: f64 = 182.0;
    const TOPIC_X: f64 = 20.0;
    const TOPIC_Y: f64 = 20.0;
    const TOPIC_WIDTH: f64 = 232.0;
    const RESOLVE_X: f64 = 340.0;
    const RESOLVE_HALF: f64 = 38.0;
    const SPINE_X: f64 = 424.0;
    const OUT_X: f64 = 448.0;
    const OUT_WIDTH: f64 = 386.0;
    const OUT_HEIGHT: f64 = 60.0;
    const LOOP_Y: f64 = 328.0;

    let mut s = String::new();
    s += &node(
        TOPIC_X,
        TOPIC_Y,
        TOPIC_WIDTH,
        64.0,
        ("research-topic", "cited, on research/<topic>"),
        "box",
        "tm",
        None,
    );
    s += &arrow(
        TOPIC_X + TOPIC_WIDTH / 2.0,
        TOPIC_Y + 64.0,
        TOPIC_X + TOPIC_WIDTH / 2.0,
        CENTER_Y - 34.0,
        false,
    );
    s += &text(
        TOPIC_X + TOPIC_WIDTH / 2.0 + 12.0,
        CENTER_Y - 60.0,
        "PR merges",
        "ts",
        "start",
    );
    s += &node(
        TOPIC_X,
        CENTER_Y - 32.0,
        TOPIC_WIDTH,
        64.0,
        ("research/<topic>.md", "on record, unresolved"),
        "store",
        "tm",
        None,
    );
    s += &arrow(
        TOPIC_X + TOPIC_WIDTH,
        CENTER_Y,
        RESOLVE_X - RESOLVE_HALF - 2.0,
        CENTER_Y,
        false,
    );
    s += &diamond(RESOLVE_X, CENTER_Y, RESOLVE_HALF, "decision");
    s += &text(
        RESOLVE_X,
        CENTER_Y - RESOLVE_HALF - 14.0,
        "resolve-research",
        "tm",
        "middle",
    );
    s += &text(RESOLVE_X, CENTER_Y + 4.0, "own PR", "ts", "middle");
    let _ = write!(
        s,
        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"edge\"/>",
        n(RESOLVE_X + RESOLVE_HALF),
        n(CENTER_Y),
        n(SPINE_X),
        n(CENTER_Y)
    );

    let mut y0 = CENTER_Y - OUT_HEIGHT - 16.0 - OUT_HEIGHT / 2.0;
    for (i, outcome) in OUTCOMES.iter().enumerate() {
        let cy = y0 + OUT_HEIGHT / 2.0;
        s += &path_arrow(
            &format!(
                "M{} {}V{}H{}",
                n(SPINE_X),
                n(CENTER_Y),
                n(cy),
                n(OUT_X - 2.0)
            ),
            false,
        );
        let (class, accent) = if i == 0 {
            ("box", Some("var(--accent)"))
        } else {
            ("ext", None)
        };
        s += &node(
            OUT_X, y0, OUT_WIDTH, OUT_HEIGHT, *outcome, class, "tb", accent,
        );
        y0 += OUT_HEIGHT + 16.0;
    }

    // The remainder waits for vet-source, then goes round again.
    let partial_bottom = CENTER_Y + OUT_HEIGHT + 16.0 + OUT_HEIGHT / 2.0;
    s += &path_arrow(
        &format!(
            "M{} {}V{}H{}V{}",
            n(OUT_X + OUT_WIDTH / 2.0),
            n(partial_bottom),
            n(LOOP_Y),
            n(RESOLVE_X),
            n(CENTER_Y + RESOLVE_HALF + 2.0)
        ),
        true,
    );
    s += &text(
        f64::midpoint(RESOLVE_X, OUT_X + OUT_WIDTH / 2.0),
        LOOP_Y - 8.0,
        "once vet-source clears its source",
        "ts",
        "middle",
    );

    document(
        OUT_X + OUT_WIDTH + 20.0,
        LOOP_Y + 28.0,
        TITLE,
        DESCRIPTION,
        &s,
    )
}

// The design system's tokens: bg, surface, raised, line, ink, muted, brand, house, and the
// three lane-* tokens. Copy a changed value from
// https://claude.ai/artifact/QfYm5d57sbJxs3W3jMyGiy, as the website repository's
// awesome-humans.css does. The Rust accent stays out: the design system shows an ecosystem's
// accent only in four places on the site, and a diagram isn't one. Brand marks the CI gate and
// the outcome the agents work towards, and house what the owner decides. Each lane takes its
// lane-* token, chosen to stay apart from those and from stale.
const TOKENS_LIGHT: &str = "--bg: #ffffff; --surface: #f6f7f8; --band: #eef0f2; --fg: #0d1117; --muted: #5b6572; --line: #e3e6ea;
  --accent: #087d4e; --human: #9a6100; --auto: #087d4e;
  --sources: #017b80; --research: #6d7123; --opinions: #955080; --reports: #8253ac;";

const TOKENS_DARK: &str = "--bg: #0d1117; --surface: #151a21; --band: #1c232c; --fg: #e8ecf0; --muted: #8b95a1; --line: #2a323c;
  --accent: #3dd392; --human: #f2c14e; --auto: #3dd392;
  --sources: #6ac9ce; --research: #b9be71; --opinions: #e79bce; --reports: #c9a5e9;";

// Font families are literal rather than custom properties, and set apart from size and weight
// rather than through the font shorthand: a browser that rejects either form drops the whole
// declaration and draws every label in its default font, which is wider than the layout allows
// for. system-ui is left out because privacy-hardened browsers restrict it. Each stack names
// the design system's IBM Plex first, then a common font per platform before the generic
// family. An SVG shown as an image can't load a web font, so most readers see a fallback.
const FONTS: &str = r#".t, .tb, .ts, .gate-t, .check-t { font-family: "IBM Plex Sans", "Segoe UI", "Noto Sans", "DejaVu Sans", "Helvetica Neue", Arial, sans-serif; }
.tm, .th { font-family: "IBM Plex Mono", "Cascadia Mono", Consolas, "Noto Sans Mono", "DejaVu Sans Mono", Menlo, monospace; }
"#;

const CLASSES: &str = ".bg { fill: var(--bg); }
.box { fill: var(--surface); stroke: var(--line); stroke-width: 1.2; }
.chip { fill: var(--band); stroke: none; }
.ext { fill: var(--band); stroke: var(--muted); stroke-width: 1.2; }
.store { fill: var(--band); stroke: var(--line); stroke-width: 1.2; }
.t { fill: var(--fg); font-weight: 500; font-size: 16px; }
.tb { fill: var(--fg); font-weight: 600; font-size: 16px; }
.tm { fill: var(--fg); font-weight: 600; font-size: 15px; }
.ts { fill: var(--muted); font-weight: 400; font-size: 14px; }
.th { fill: var(--muted); font-weight: 600; font-size: 13px; letter-spacing: .08em; text-transform: uppercase; }
.edge { stroke: var(--muted); stroke-width: 1.4; fill: none; }
.edge.dash { stroke-dasharray: 4 4; }
.arrowhead { fill: var(--muted); }
.gate { fill: var(--surface); stroke: var(--human); stroke-width: 2; }
.gate-t { fill: var(--human); font-weight: 600; font-size: 14px; }
.check { fill: var(--surface); stroke: var(--auto); stroke-width: 2; }
.check-t { fill: var(--auto); font-weight: 600; font-size: 14px; }
.decision { fill: var(--surface); stroke: var(--accent); stroke-width: 2; }";

const MARKER: &str = r#"<defs><marker id="ah" viewBox="0 0 10 10" refX="9" refY="5" markerWidth="7" markerHeight="7" orient="auto-start-reverse"><path class="arrowhead" d="M0 0L10 5L0 10z"/></marker></defs>"#;

// Wraps the drawing in a standalone, accessible SVG document with its styles and background.
fn document(width: f64, height: f64, title: &str, description: &str, body: &str) -> String {
    let (width, height) = (n(width), n(height));
    format!(
        "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg role=\"img\" aria-labelledby=\"title desc\" xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {width} {height}\"><title id=\"title\">{}</title><desc id=\"desc\">{}</desc><style>\nsvg {{ {TOKENS_LIGHT} }}\n@media (prefers-color-scheme: dark) {{ svg {{ {TOKENS_DARK} }} }}\n\n{FONTS}{CLASSES}\n\n</style>{MARKER}<rect width=\"{width}\" height=\"{height}\" rx=\"10\" class=\"bg\"/>{body}</svg>\n",
        escape(title),
        escape(description)
    )
}

// A titled box with a subtitle and an optional coloured edge on its left.
#[expect(
    clippy::too_many_arguments,
    reason = "a box's position, size, text, and styles, as the .NET helper takes them"
)]
fn node(
    x: f64,
    y: f64,
    width: f64,
    height: f64,
    (title, subtitle): (&str, &str),
    class: &str,
    title_class: &str,
    accent: Option<&str>,
) -> String {
    let edge = accent.map_or_else(String::new, |fill| accent_edge(x, y, height, fill));
    let cy = y + height / 2.0;
    rect(x, y, width, height, 8.0, class)
        + &edge
        + &text(x + 16.0, cy - 4.0, title, title_class, "start")
        + &text(x + 16.0, cy + 17.0, subtitle, "ts", "start")
}

fn accent_edge(x: f64, y: f64, height: f64, fill: &str) -> String {
    format!(
        "<rect x=\"{}\" y=\"{}\" width=\"4\" height=\"{}\" rx=\"2\" fill=\"{fill}\"/>",
        n(x),
        n(y),
        n(height)
    )
}

fn diamond(cx: f64, cy: f64, half: f64, class: &str) -> String {
    format!(
        "<path d=\"M{} {}L{} {}L{} {}L{} {}z\" class=\"{class}\"/>",
        n(cx),
        n(cy - half),
        n(cx + half),
        n(cy),
        n(cx),
        n(cy + half),
        n(cx - half),
        n(cy)
    )
}

fn text(x: f64, y: f64, content: &str, class: &str, anchor: &str) -> String {
    format!(
        "<text x=\"{}\" y=\"{}\" class=\"{class}\" text-anchor=\"{anchor}\">{}</text>",
        n(x),
        n(y),
        escape(content)
    )
}

// Lines of text, vertically centred on y.
fn lines(x: f64, y: f64, items: &[&str]) -> String {
    const STEP: f64 = 19.0;
    let count = f64::from(u32::try_from(items.len()).unwrap_or(1));
    let mut line_y = y - (count - 1.0) * STEP / 2.0 + 5.0;
    let mut out = String::new();
    for item in items {
        out += &text(x, line_y, item, "t", "start");
        line_y += STEP;
    }
    out
}

fn rect(x: f64, y: f64, width: f64, height: f64, radius: f64, class: &str) -> String {
    format!(
        "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" rx=\"{}\" class=\"{class}\"/>",
        n(x),
        n(y),
        n(width),
        n(height),
        n(radius)
    )
}

fn arrow(x1: f64, y1: f64, x2: f64, y2: f64, dash: bool) -> String {
    format!(
        "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" class=\"{}\" marker-end=\"url(#ah)\"/>",
        n(x1),
        n(y1),
        n(x2),
        n(y2),
        edge_class(dash)
    )
}

fn path_arrow(d: &str, dash: bool) -> String {
    format!(
        "<path d=\"{d}\" class=\"{}\" marker-end=\"url(#ah)\"/>",
        edge_class(dash)
    )
}

/// A coordinate with at most two decimals and no trailing zeros, as the .NET port's `0.##`.
fn n(value: f64) -> String {
    let text = format!("{value:.2}");
    let text = text.trim_end_matches('0').trim_end_matches('.');
    if text == "-0" {
        "0".into()
    } else {
        text.into()
    }
}

fn edge_class(dash: bool) -> &'static str {
    if dash { "edge dash" } else { "edge" }
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n_drops_trailing_zeros() {
        assert_eq!(n(12.0), "12");
        assert_eq!(n(12.5), "12.5");
        assert_eq!(n(1.0 / 3.0), "0.33");
    }

    #[test]
    fn every_diagram_is_a_titled_svg_document() {
        for (svg, title) in [
            (pipeline(), "Maintenance pipeline"),
            (standing(), "Source standing"),
            (research(), "Research lifecycle"),
        ] {
            assert!(svg.starts_with("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<svg "));
            assert!(svg.contains(&format!("<title id=\"title\">{title}</title>")));
            assert!(svg.ends_with("</svg>\n"));
        }
    }

    #[test]
    fn the_pipeline_names_the_rust_refresh() {
        let svg = pipeline();
        assert!(svg.contains("refresh-rust-versions"));
        assert!(svg.contains("refresh/rust-&lt;version&gt;"));
        assert!(!svg.contains("dotnet"));
    }
}
