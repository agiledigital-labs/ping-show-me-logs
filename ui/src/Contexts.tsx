import { createContext } from "react";

export const AppSharedContext = createContext<{ location: string }>({
  location: "web",
});
