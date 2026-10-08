---
name: Rusty Pomodoro website
description: A quiet studio practice sheet for focused desktop work.
colors:
  primary: "#bd3c2c"
  primary-hover: "#9c2e22"
  paper: "#f6f3eb"
  ink: "#292d27"
  muted: "#616359"
  clay: "#e8d7c5"
  green: "#233c32"
  green-text: "#d1d8c9"
  rule: "#d6d5c9"
typography:
  display:
    fontFamily: "Fraunces, Georgia, serif"
    fontSize: "clamp(56px, 6.7vw, 96px)"
    fontWeight: 500
    lineHeight: 1.07
    letterSpacing: "-0.035em"
  headline:
    fontFamily: "Fraunces, Georgia, serif"
    fontSize: "clamp(40px, 4.1vw, 60px)"
    fontWeight: 500
    lineHeight: 1.07
  body:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.6
  title:
    fontFamily: "DM Sans, sans-serif"
    fontSize: "23px"
    fontWeight: 500
    lineHeight: 1.35
rounded:
  control: "5px"
  surface: "8px"
spacing:
  group: "24px"
  section-desktop: "112px"
  section-mobile: "72px"
components:
  button-primary:
    backgroundColor: "{colors.primary}"
    textColor: "{colors.paper}"
    rounded: "{rounded.control}"
    padding: "14px 22px"
  button-primary-hover:
    backgroundColor: "{colors.primary-hover}"
  button-light:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.green}"
    rounded: "{rounded.control}"
    padding: "14px 22px"
---

# Design System: Rusty Pomodoro website

## Overview

**Creative North Star: "The studio practice sheet"**

An editorial website with generous room around concise instructions, confident serif headlines, and quiet sans-serif detail. Flat paper, clay and green fields carry real app screenshots rather than decorative imagery. This is the website's visual system, not a redesign of the desktop app.

**Key Characteristics:**
- Flat, deliberately separated color fields.
- Serif headlines and restrained sans-serif body text.
- Real app imagery and direct installation actions.

## Colors

### Primary

Vermilion marks primary actions, selected screenshot views, and headline emphasis.

### Secondary

Studio green carries the browser timer. Clay provides a warm screenshot ground.

### Neutral

Paper is the reading field. Ink carries headlines and body text; muted ink carries supporting copy. Rules separate information without card containers.

**The One Action Rule.** Use vermilion for the action or emphasis that should lead a group.

## Typography

Self-hosted Fraunces carries display and headline roles; self-hosted DM Sans carries body, titles, navigation and controls. Monospace is reserved for executable build commands. Timer numerals are tabular.

Body text is bounded at 70ch; short supporting sections use narrower measures. Metadata never falls below 12px.

**The Quiet Detail Rule.** Put product facts in readable body text, not promotional labels above headings.

## Layout

The container caps at 1240px. Desktop gutters total 112px; at 1100px they total 72px, at 800px they total 48px, and below 380px they total 36px. Editorial columns collapse at 800px. Section spacing changes from 112px to 72px. Longer installation commands scroll inside their own blocks instead of widening the page.

## Elevation & Depth

Flat fields and thin rules supply hierarchy. No CSS shadows are applied to website components. Shadows visible in the app screenshots belong to the photographed desktop window.

## Shapes

Controls use small softened corners; screenshot surfaces use restrained corners. The hero's arched clay field is a website-specific framing device, not a universal container.

## Components

Primary buttons use vermilion and paper; timer buttons invert to paper and green. Links have visible underlines. Focus uses a 3px outline, offset by 6px, with a light variant on green.

Screenshot selectors are ruled rows with pressed state and an authored short arrow transition. Timer phases use compact selected outlines. FAQ rows use native details/summary. All demo controls remain hidden until JavaScript initializes.

Motion consists of state transitions and smooth anchor travel; reduced motion disables both. Content is never hidden by entrance animation.

## Do's and Don'ts

### Do:
- Do show actual app screenshots with descriptive alternate text.
- Do use flat fields and consistent spacing to separate information.
- Do keep focus indicators and reduced-motion behavior.

### Don't:
- Don't fabricate release availability, customer metrics or statistics.
- Don't replace product evidence with decorative card grids.
- Don't apply this website's system to the native desktop UI.
