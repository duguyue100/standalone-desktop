/**
 * AlfAlfa session initialization.
 *
 * After a new session is created, this module injects:
 * 1. `lf skills` output (LatticeFlow CLI knowledge base)
 * 2. Project journal overview (if meaningful content exists)
 *
 * Both are injected as noReply messages so they become invisible context.
 */

import type { Platform } from "@/context/platform"

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type SdkClient = any

const JOURNAL_JUST_INITIALIZED = "This journal has just been initialized"

/**
 * Called after a new session is created.
 * Injects lf skills and journal overview as silent context.
 */
export async function initializeAlfalfaSession(
  client: SdkClient,
  sessionId: string,
  directory: string,
  platform: Platform,
) {
  // 1. Inject lf skills
  await injectLfSkills(client, sessionId, directory, platform)

  // 2. Inject journal overview
  await injectJournalOverview(client, sessionId, directory, platform)
}

async function injectLfSkills(client: SdkClient, sessionId: string, directory: string, platform: Platform) {
  try {
    const skills = platform.loadLfSkills ? await platform.loadLfSkills() : ""
    if (skills && skills.trim().length > 0) {
      await client.session.promptAsync({
        sessionID: sessionId,
        noReply: true,
        parts: [{ type: "text", text: skills }],
        directory,
      })
    }
  } catch {
    // lf not available -- inject fallback
    try {
      await client.session.promptAsync({
        sessionID: sessionId,
        noReply: true,
        parts: [
          {
            type: "text",
            text: "Note: `lf skills` could not be loaded. You may have limited knowledge of LF CLI specifics. Use `lf --help` and `lf <command> --help` to discover available commands.",
          },
        ],
        directory,
      })
    } catch {
      // Best-effort
    }
  }
}

async function injectJournalOverview(client: SdkClient, sessionId: string, directory: string, platform: Platform) {
  try {
    // Ensure journal exists for this project
    if (platform.ensureProjectJournal) {
      await platform.ensureProjectJournal(directory)
    }

    // Read overview
    const overview = platform.getJournalOverview ? await platform.getJournalOverview(directory) : ""
    if (
      overview &&
      overview.includes("# Project Overview") &&
      !overview.includes(JOURNAL_JUST_INITIALIZED)
    ) {
      await client.session.promptAsync({
        sessionID: sessionId,
        noReply: true,
        parts: [
          {
            type: "text",
            text: `Project journal overview (for context, do not respond to this):\n\n${overview}`,
          },
        ],
        directory,
      })
    }
  } catch {
    // Journal not available -- not critical
  }
}
