import { useState } from "react";
import useSWR from "swr";
import { simpleJsonFetcher } from "./helpers.ts";

const WatchLogs = () => {
  const [watching, setWatching] = useState<string>("Error");
  const { data: watchData } = useSWR(
    `${document.URL.includes("5173") ? "http://localhost:8081" : ""}/api/logs/watch?filters=${watching ?? "All"}`,
    simpleJsonFetcher,
  );
  return (
    <>
      <h3>Watching</h3>
      <label>logs types to keep</label>
      <select
        value={watching}
        onChange={(event) => setWatching(event.target.value)}
      >
        <option value={"Warn"}>Warn</option>
        <option value={"All"}>All</option>
        <option value={"Error"}>Error</option>
        <option value={"Debug"}>Debug</option>
      </select>
      <ol>
        {watchData?.result.map((res) => (
          <li key={res.payload.transaction_id}>{res.payload.message}</li>
        ))}
      </ol>
    </>
  );
};

export default WatchLogs;
