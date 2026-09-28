// @milkdown/crepe@7.21.2 ships an index.d.ts that only re-exports
// CrepeFeature, even though the runtime ESM bundle exports Crepe,
// CrepeBuilder, and useCrepe too (verified in lib/esm/index.js). This
// augments the module's public type surface to what we actually use;
// remove once an upstream release fixes lib/types/index.d.ts.
declare module "@milkdown/crepe" {
  export enum CrepeFeature {
    ImageBlock = "image-block",
  }

  export interface CrepeConfig {
    root?: Node | string | null;
    defaultValue?: string;
    features?: Partial<Record<string, boolean>>;
    featureConfigs?: Record<string, unknown>;
  }

  export class Crepe {
    constructor(config?: CrepeConfig);
    create(): Promise<unknown>;
    destroy(): Promise<unknown>;
    getMarkdown(): string;
    setReadonly(value: boolean): this;
    on(
      fn: (api: {
        markdownUpdated(
          cb: (ctx: unknown, markdown: string, prevMarkdown: string) => void,
        ): void;
      }) => void,
    ): this;
  }
}
