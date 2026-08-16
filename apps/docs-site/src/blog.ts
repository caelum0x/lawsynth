import type { MarkdownDocument } from "./markdown.js";
export interface BlogPost {
  readonly slug: string; readonly title: string; readonly description: string; readonly publishedAt: string;
  readonly updatedAt?: string; readonly authors: readonly string[]; readonly tags: readonly string[];
  readonly document: MarkdownDocument; readonly draft?: boolean;
}
export class BlogCatalog {
  readonly posts: readonly BlogPost[];
  constructor(posts: readonly BlogPost[], includeDrafts = false) {
    const slugs = new Set<string>();
    this.posts = Object.freeze(posts.filter((post) => includeDrafts || post.draft !== true).map((post) => {
      if (!/^[a-z0-9][a-z0-9-]{1,100}$/u.test(post.slug) || slugs.has(post.slug)) throw new RangeError(`invalid blog slug: ${post.slug}`);
      if (!post.title.trim() || !post.description.trim() || !Number.isFinite(Date.parse(post.publishedAt))) throw new RangeError(`invalid blog post: ${post.slug}`);
      if (post.authors.length === 0 || post.authors.some((author) => !author.trim())) throw new RangeError(`blog post ${post.slug} needs authors`);
      slugs.add(post.slug); return Object.freeze(post);
    }).sort((a, b) => b.publishedAt.localeCompare(a.publishedAt)));
  }
  get(slug: string): BlogPost | undefined { return this.posts.find((post) => post.slug === slug); }
  byTag(tag: string): readonly BlogPost[] { return Object.freeze(this.posts.filter((post) => post.tags.includes(tag))); }
  archive(): ReadonlyMap<string, readonly BlogPost[]> {
    const groups = new Map<string, BlogPost[]>();
    for (const post of this.posts) { const year = post.publishedAt.slice(0, 4); groups.set(year, [...(groups.get(year) ?? []), post]); }
    return groups;
  }
}
