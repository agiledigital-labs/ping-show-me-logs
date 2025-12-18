import type { Fetcher } from "swr";
import type { Root } from "../types";

export const jsonFetcher = (url: string) => fetch(url).then((r) => r.json());

export const simpleJsonFetcher: Fetcher<Root, string> = (url: string) =>
  fetch(url).then((r) => r.json());
