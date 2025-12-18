import { useState } from "react";
import useSWR from "swr";
import { simpleJsonFetcher } from "./helpers";

const SearchLogs = () => {
  const [frRequestId, setFrRequestId] = useState<string>();
  const { data } = useSWR(
    frRequestId === undefined
      ? null
      : `${document.URL.includes("5173") ? "http://localhost:8081" : ""}/api/logs/${frRequestId}?filters=Error`,
    simpleJsonFetcher,
  );
  return (
    <>
      <h3>From Form</h3>
      <form className="card">
        <label htmlFor={"frId"}>Fr request id </label>
        <input
          id={"frId"}
          onChange={(event) => setFrRequestId(event.target.value)}
        />
      </form>
      <ol>
        {/* eslint-disable-next-line @typescript-eslint/no-explicit-any*/}
        {data?.result.map((res: any, idx: number) => (
          <li key={res.id ?? res.payload?.id ?? idx}>{res.payload.message}</li>
        ))}
      </ol>
    </>
  );
};

export default SearchLogs;
