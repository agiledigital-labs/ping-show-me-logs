import { StrictMode } from "react";
import { createRoot } from "react-dom/client";
import "./index.css";
import AppNav from "./AppNav.tsx";
import { BrowserRouter, Routes, Route } from "react-router";
import { Flow, SearchLogs, WatchLogs } from "./pages";

createRoot(document.getElementById("root")!).render(
  <StrictMode>
    <BrowserRouter>
      <AppNav>
        <Routes>
          <Route index element={<Flow />} />
          <Route path={"search"} element={<SearchLogs />} />
          <Route path={"watch"} element={<WatchLogs />} />
        </Routes>
      </AppNav>
    </BrowserRouter>
  </StrictMode>,
);
