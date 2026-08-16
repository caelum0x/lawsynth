export interface SearchDocument { readonly id: string; readonly path: string; readonly title: string; readonly text: string; readonly headings?: readonly string[]; readonly tags?: readonly string[]; readonly version?: string; }
export interface SearchHit { readonly document: SearchDocument; readonly score: number; readonly excerpt: string; readonly terms: readonly string[]; }

const STOP_WORDS = new Set(["a", "an", "and", "are", "as", "at", "be", "by", "for", "from", "in", "is", "it", "of", "on", "or", "that", "the", "to", "with"]);
function tokens(value: string): string[] { return value.toLocaleLowerCase().normalize("NFKD").split(/[^\p{L}\p{N}_-]+/u).filter((token) => token.length > 1 && !STOP_WORDS.has(token)); }

export class SearchIndex {
  #documents = new Map<string, SearchDocument>();
  #postings = new Map<string, Map<string, number>>();

  add(document: SearchDocument): void {
    if (!document.id.trim() || !document.path.startsWith("/") || !document.title.trim() || this.#documents.has(document.id)) throw new RangeError(`invalid or duplicate search document: ${document.id}`);
    const frozen = Object.freeze({ ...document });
    this.#documents.set(document.id, frozen);
    const weighted = [document.title, document.title, ...(document.headings ?? []), ...(document.tags ?? []), document.text];
    const frequencies = new Map<string, number>();
    for (const term of tokens(weighted.join(" "))) frequencies.set(term, (frequencies.get(term) ?? 0) + 1);
    for (const [term, count] of frequencies) {
      const posting = this.#postings.get(term) ?? new Map<string, number>();
      posting.set(document.id, count); this.#postings.set(term, posting);
    }
  }

  search(query: string, options: { readonly limit?: number; readonly version?: string; } = {}): readonly SearchHit[] {
    const terms = [...new Set(tokens(query))];
    const limit = options.limit ?? 20;
    if (!Number.isInteger(limit) || limit < 1 || limit > 100) throw new RangeError("search limit must be in 1..100");
    if (terms.length === 0) return [];
    const scores = new Map<string, number>();
    for (const term of terms) {
      const matches = this.#postings.get(term);
      if (matches === undefined) continue;
      const inverse = Math.log(1 + this.#documents.size / matches.size);
      for (const [id, count] of matches) scores.set(id, (scores.get(id) ?? 0) + (1 + Math.log(count)) * inverse);
    }
    return Object.freeze([...scores].map(([id, score]) => {
      const document = this.#documents.get(id)!;
      return { document, score, excerpt: excerpt(document.text, terms), terms: Object.freeze(terms) };
    }).filter((hit) => options.version === undefined || hit.document.version === options.version)
      .sort((left, right) => right.score - left.score || left.document.title.localeCompare(right.document.title)).slice(0, limit));
  }
}

function excerpt(text: string, terms: readonly string[], length = 180): string {
  const lower = text.toLocaleLowerCase();
  const position = Math.min(...terms.map((term) => lower.indexOf(term)).filter((index) => index >= 0));
  if (!Number.isFinite(position)) return text.slice(0, length).trim();
  const start = Math.max(0, position - Math.floor(length / 3));
  return `${start > 0 ? "…" : ""}${text.slice(start, start + length).trim()}${start + length < text.length ? "…" : ""}`;
}
