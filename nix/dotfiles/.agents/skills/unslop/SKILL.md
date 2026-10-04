---
name: unslop
description: Cut AI tells from text you write or edit for a human reader (commit messages, PR titles and bodies, docs, code comments, replies). Apply before committing, posting, or sending; leave prose you didn't touch alone.
---

# Unslop

Edit text to remove AI patterns and add human voice.

## Process

1. Scan for the patterns below.
2. Rewrite. Preserve meaning, match intended tone.
3. Add soul (see next section).
4. Self-audit: "What makes this obviously AI generated?" Fix remaining tells.

## Adding soul

- **Have opinions.** React to facts instead of neutrally listing pros and cons.
- **Vary rhythm.** Short sentences. Then longer ones. Mix it up.
- **Acknowledge complexity.** "Impressive but also kind of unsettling" beats "impressive."
- **Use "I" when it fits.** First person isn't unprofessional.
- **Let some mess in.** Perfect structure looks machine-made.
- **Be specific.** Not "this is concerning" but "there's something unsettling about agents churning away at 3am."

## Patterns to detect and fix

### Content

1. **Puffery.** "pivotal moment", "testament to", "evolving landscape". Cut, state what happened.
2. **Name-dropping.** Listing media outlets without context. Pick one, say what was said.
3. **Superficial -ing phrases.** "highlighting...", "ensuring...", "showcasing...". Delete or expand with real sources.
4. **Promotional language.** "nestled", "vibrant", "breathtaking", "groundbreaking". Use neutral descriptions.
5. **Vague attributions.** "Experts believe". Name the source or delete.
6. **Formulaic challenges.** "Despite challenges... continues to thrive." Replace with specific facts.

### Language

7. **AI vocabulary.** Additionally, crucial, delve, enduring, enhance, fostering, garner, interplay, intricate, landscape (abstract), pivotal, showcase, tapestry (abstract), testament, underscore, vibrant. Replace with plain words.
8. **Fancy ways to say "is".** "serves as", "stands as", "boasts", "features". Say "is" or "has".
9. **"Not just X, but Y."** State the point directly.
10. **Rule of three.** Forcing groups of three. Use the natural number.
11. **Synonym cycling.** Protagonist, main character, central figure, hero in one paragraph. Pick one, repeat it.
12. **False ranges.** "from X to Y" where X and Y aren't on a meaningful scale. List topics directly.

### Style

13. **Em dash overuse.** Avoid em dashes entirely. Use periods or commas only. If a thought needs separation, end the sentence or use a comma.
14. **Colon overuse.** Fine before a list or example. Not as mid-sentence connectors. Rewrite to let the point stand.
15. **Boldface overuse.** Don't bold every proper noun or acronym.
16. **Inline-header lists.** Bold label + colon restating the line: "**Performance:** Performance improved...". Convert to prose. A bold lead-in ending in a period with new detail is fine.
17. **Title case headings.** Use sentence case.
18. **Decorative emojis.** Remove from headings and bullets.
19. **Curly quotes.** Replace with straight quotes.

### Communication artifacts

20. **Chatbot phrases.** "I hope this helps!", "Let me know if...", "Of course!", "Certainly!". Remove.
21. **Cutoff disclaimers.** "While specific details are limited...". Find sources or remove.
22. **Sycophantic tone.** "Great question! You're absolutely right!" Respond directly.

### Filler

23. **Filler phrases.** "In order to" → "To". "Due to the fact that" → "Because". "It is important to note that" → delete.
24. **Excessive hedging.** "could potentially possibly be argued that it might" → "may".
25. **Generic conclusions.** "The future looks bright." State specific plans or facts.

### Jargon

26. **Abstract metaphor nouns.** Substrate, wedge, vector, locus, nexus, harness (as metaphor), surface (as "API surface"), bedrock, scaffolding, modality, paradigm, gold-plating, ratchet, evacuate, endgame, north star, flywheel. Pick the concrete word: "Substrate" → "base". "Wedge in" → "add". "Vector" → "way" or "method".

### Plain speech

27. **Say what it does, not how it feels.** Name the mechanism or a number. If it could appear unchanged in another project's docs, cut it.
28. **Shorten or split dense sentences.** One idea per sentence.
29. **Active voice.** "queries are validated" → "the compiler validates queries". Passive only when actor is unknown or irrelevant.
30. **Cut adverbs, or use a stronger verb.** "runs quickly" → "is fast" or the number. An adverb propping up a weak verb means the verb is wrong.
31. **Prefer the plain word.** "utilize" → "use". "leverage" → "use". "facilitate" → "help". "numerous" → "many". "in the event that" → "if".
