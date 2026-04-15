/**
 * Journal service for AlfAlfa.
 *
 * Handles /journal (update project journal from current session) and
 * /journal-review (review journal for gaps, stale info, contradictions).
 *
 * Creates a temporary session and sends the journal prompt. The journal
 * lives at <project>/.lf_agent/journal/ so it's inside the project
 * directory and doesn't require extra file permissions.
 */

// eslint-disable-next-line @typescript-eslint/no-explicit-any
type SdkClient = any

// ---------------------------------------------------------------------------
// Prompts
// ---------------------------------------------------------------------------

const JOURNAL_PROMPT = `Based on the session activity summary below, update the project journal following the conventions in the journal schema.

Specifically:
1. Read .lf_agent/journal/_schema.md to understand conventions
2. Read .lf_agent/journal/_log.md and .lf_agent/journal/overview.md for current state
3. Create or update relevant topic pages in .lf_agent/journal/ based on what was done in the session
4. Append a new entry to .lf_agent/journal/_log.md with today's date and a summary
5. Update .lf_agent/journal/overview.md if the project state has meaningfully changed

Focus only on the user's actual work — evaluations created, issues found, decisions made, ideas discussed.`

const REVIEW_PROMPT = `Review the project journal at .lf_agent/journal/.

1. Read .lf_agent/journal/_schema.md to understand conventions
2. Read .lf_agent/journal/_log.md and .lf_agent/journal/overview.md
3. Read all other journal pages (use glob ".lf_agent/journal/*.md" to find them, then read each)
4. Report:
   - Pages that may be stale (not updated recently but referenced in recent work)
   - Gaps: important topics mentioned but lacking their own page
   - Contradictions between pages
   - Suggested next investigations or evaluations to try
5. Update any pages that need corrections
6. Update .lf_agent/journal/overview.md with your findings if appropriate`

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function truncate(text: string, maxLen: number): string {
  if (text.length <= maxLen) return text
  return text.slice(0, maxLen) + "\u2026"
}

interface SessionMessage {
  info: { role: string }
  parts?: Array<{ type: string; text?: string }>
}

/**
 * Build a markdown summary of a session's conversation.
 */
async function buildSessionSummary(client: SdkClient, sessionId: string, directory: string): Promise<string> {
  const result = await client.session.messages({ sessionID: sessionId, directory })
  const messages: SessionMessage[] = result?.data ?? []
  if (messages.length === 0) return "(empty session)"

  const lines: string[] = []
  for (const msg of messages) {
    const role = msg.info.role
    const textParts = (msg.parts ?? [])
      .filter((p: { type: string }) => p.type === "text")
      .map((p: { text?: string }) => p.text ?? "")
      .filter((t: string) => t.length > 0)

    if (textParts.length === 0) continue
    const text = truncate(textParts.join("\n"), 500)
    if (role === "user") lines.push(`**User:** ${text}\n`)
    else if (role === "assistant") lines.push(`**Alfalfa:** ${text}\n`)
  }
  return lines.length > 0 ? lines.join("\n") : "(empty session)"
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export interface JournalRunResult {
  success: boolean
  error?: string
}

/**
 * Run /journal: build a summary of the current session, create a temp session,
 * and send the combined context + journal prompt.
 *
 * Uses promptAsync which starts the agent working and returns immediately.
 * The temp session persists as a record of the journal update.
 */
export async function runJournal(
  client: SdkClient,
  mainSessionId: string,
  _journalPath: string,
  directory: string,
): Promise<JournalRunResult> {
  try {
    const summary = await buildSessionSummary(client, mainSessionId, directory)
    const created = await client.session.create({ directory })
    const tempSessionId = created?.data?.id
    if (!tempSessionId) throw new Error("Failed to create temporary session")

    const fullPrompt = `## Session Summary\n\n${summary}\n\n---\n\n${JOURNAL_PROMPT}`
    await client.session.promptAsync({
      sessionID: tempSessionId,
      parts: [{ type: "text", text: fullPrompt }],
      directory,
    })

    return { success: true }
  } catch (e) {
    return {
      success: false,
      error: e instanceof Error ? e.message : String(e),
    }
  }
}

/**
 * Run /journal-review: create a temp session and instruct the agent to
 * review all journal pages for gaps, stale info, and contradictions.
 */
export async function runJournalReview(
  client: SdkClient,
  _journalPath: string,
  directory: string,
): Promise<JournalRunResult> {
  try {
    const created = await client.session.create({ directory })
    const tempSessionId = created?.data?.id
    if (!tempSessionId) throw new Error("Failed to create temporary session")

    await client.session.promptAsync({
      sessionID: tempSessionId,
      parts: [{ type: "text", text: REVIEW_PROMPT }],
      directory,
    })

    return { success: true }
  } catch (e) {
    return {
      success: false,
      error: e instanceof Error ? e.message : String(e),
    }
  }
}
