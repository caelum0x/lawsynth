import type { Point } from "./layout.js";
export type Easing = "linear" | "easeInOut" | "easeOut";
export function ease(t: number, easing: Easing = "easeInOut"): number { if (!Number.isFinite(t)) throw new RangeError("animation progress must be finite"); const x=Math.min(1,Math.max(0,t)); return easing === "linear" ? x : easing === "easeOut" ? 1-(1-x)*(1-x) : x*x*(3-2*x); }
export function interpolate(from: number, to: number, progress: number, easing: Easing = "easeInOut"): number { return from+(to-from)*ease(progress,easing); }
export function interpolatePoint(from: Point, to: Point, progress: number, easing: Easing = "easeInOut"): Point { return {x:interpolate(from.x,to.x,progress,easing),y:interpolate(from.y,to.y,progress,easing)}; }
export interface AnimationFrame<T> { readonly value: T; readonly done: boolean; }
export function animationFrame<T>(from: T, to: T, now: number, start: number, duration: number, interpolateValue: (from:T,to:T,progress:number)=>T): AnimationFrame<T> { if (duration < 0 || !Number.isFinite(now) || !Number.isFinite(start)) throw new RangeError("invalid animation timing"); const progress=duration === 0 ? 1 : Math.min(1,Math.max(0,(now-start)/duration)); return {value:interpolateValue(from,to,progress),done:progress===1}; }
