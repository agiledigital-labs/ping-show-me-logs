import {
  Box,
  FormControl,
  Grid,
  InputLabel,
  MenuItem,
  Select,
  TextField,
} from "@mui/material";
import {
  ReactFlow,
  ReactFlowProvider,
  useOnSelectionChange,
} from "@xyflow/react";
import { type Dispatch, useEffect, useReducer, useState } from "react";
import { Controller, useForm } from "react-hook-form";
import "./Flow.css";
import { useSearchParams } from "react-router";
import useSWR from "swr";
import * as z from "zod";
import { PingNode } from "./custom/CustomNodes.tsx";
import { Log } from "./custom/Logs.tsx";
import { jsonFetcher } from "./helpers.ts";

type Inputs = {
  selectedTree?: string;
  startsWith?: string;
  endsWith?: string;
  contains?: string;
};

const nodeTypes = {
  ping: PingNode,
};

export type State = {
  flow: Record<string, Array<string>>;
  transactions: Record<string, string[]>;
};

type ActionType = "NewTransactionId" | "BulkSetTransactionIds" | "WatchJourney";

type Action<A extends ActionType, T extends object> = { type: A; msg: T };

type TransactionAction = Action<
  "NewTransactionId",
  { journeyId: string; transactionId: string }
>;

type BulkSetTransactionIds = Action<
  "BulkSetTransactionIds",
  {
    journeyId: string;
    transactionIds: string[];
  }
>;

type WatchJourney = Action<
  "WatchJourney",
  {
    journeyId: string;
  }
>;

const makeNewTransactionIdAction = (
  journeyId: string,
  transactionId: string,
): TransactionAction => ({
  type: "NewTransactionId",
  msg: { journeyId, transactionId },
});

export type Actions = TransactionAction | BulkSetTransactionIds | WatchJourney;

const reducer = (state: State, action: Actions) => {
  switch (action.type) {
    case "WatchJourney": {
      return {
        ...state,
        flow: {
          ...state.flow,
          [action.msg.journeyId]: state.flow[action.msg.journeyId] ?? [],
        },
      };
    }
    case "NewTransactionId": {
      const transaction = action.msg.transactionId;
      const journey = action.msg.journeyId;

      return {
        ...state,
        flow: {
          ...state.flow,
          [journey]: [...(state.flow[journey] ?? []), transaction],
        },
        transactions: {
          ...state.transactions,
          [transaction]: [...(state.transactions[transaction] ?? []), journey],
        },
      };
    }
    case "BulkSetTransactionIds": {
      const transaction = action.msg.transactionIds;
      const journey = action.msg.journeyId;
      return {
        ...state,
        flow: {
          ...state.flow,
          [journey]: [...(state.flow[journey] ?? []), ...transaction],
        },
      };
    }

    default: {
      return state;
    }
  }
};

const FlowInner = ({
  state,
  dispatch,
  ws,
}: {
  state: State;
  dispatch: Dispatch<Actions>;
  ws?: WebSocket;
}) => {
  console.log(state, typeof dispatch);
  const [searchParams] = useSearchParams();
  const values = {
    startsWith: searchParams.get("startsWith") ?? undefined,
    endsWith: searchParams.get("endsWith") ?? undefined,
    container: searchParams.get("container") ?? undefined,
    selectedTree: searchParams.get("selectedJourney") ?? undefined,
  };
  const { watch, control } = useForm<Inputs>({ defaultValues: values });

  const urlSearch = new URLSearchParams({
    starts_with: watch("startsWith") ?? "",
    contains: watch("contains") ?? "",
  });

  const { data: journeyList } = useSWR(
    `${document.URL.includes("5173") ? "http://localhost:8081" : ""}/api/journey?${urlSearch.toString()}`,
    jsonFetcher,
  );

  const selectedJourney = watch("selectedTree");

  const [selectedNode, setSelectedNode] = useState<string>();
  const [transactionId, setTransactionId] = useState<string>();

  const stateForJourney = selectedJourney ? state.flow[selectedJourney] : [];
  console.log(stateForJourney);
  useEffect(() => {
    if (selectedJourney) {
      dispatch({ type: "WatchJourney", msg: { journeyId: selectedJourney } });
      ws.send(`/join ${selectedJourney}`);
    }
    const lastIndex = stateForJourney?.length - 1;
    if (lastIndex >= 0) {
      console.log(stateForJourney[lastIndex], lastIndex, stateForJourney);
      setTransactionId(stateForJourney[lastIndex]);
    }
  }, [selectedJourney, stateForJourney]);

  useOnSelectionChange({
    onChange: (data) => {
      const node = data.nodes[0];

      if (!node) {
        return;
      }

      setSelectedNode(node.id as string);
    },
  });

  const { data: journeyFlow } = useSWR(
    selectedJourney === undefined
      ? null
      : `${document.URL.includes("5173") ? "http://localhost:8081" : ""}/api/journey/${selectedJourney}/flow${transactionId !== undefined ? `?transaction_id=${transactionId}` : ""}`,
    jsonFetcher,
  );

  const { data: journeyTransactions } = useSWR(
    selectedJourney === undefined
      ? null
      : `${document.URL.includes("5173") ? "http://localhost:8081" : ""}/api/journey/${selectedJourney}/transactions`,
    (url: string) =>
      jsonFetcher(url).then(
        (
          data: {
            transaction_id: string;
            timestamp: string;
          }[],
        ) => [
          ...new Set(
            data
              .sort(({ timestamp: timestampA }, { timestamp: timestampB }) =>
                timestampA > timestampB ? 1 : -1,
              )
              .map(({ transaction_id }) =>
                transaction_id.split("-request")[0].replace(/\/\d/gu, ""),
              ),
          ),
        ],
      ),
    {
      refreshWhenHidden: false,
      revalidateOnReconnect: true,
      revalidateIfStale: false,
      revalidateOnFocus: false,
    },
  );

  const { data: journeyScripts } = useSWR(
    selectedJourney === undefined
      ? null
      : `${document.URL.includes("5173") ? "http://localhost:8081" : ""}/api/journey/${selectedJourney}/scripts`,
    jsonFetcher,
  );

  const { data: scriptLogs } = useSWR(
    !journeyScripts || !selectedNode || !transactionId
      ? null
      : `${document.URL.includes("5173") ? "http://localhost:8081" : ""}/api/logs/${transactionId}?script_id=${journeyScripts?.[selectedNode]?.find((obj: Record<string, string>) => obj["type"] === "Scirpt")?._id}&script_name=${journeyScripts?.[selectedNode]?.find((obj: Record<string, string>) => obj["type"] === "Scirpt")?.name}`,
    jsonFetcher,
  );

  const nodes = journeyFlow?.nodes.map(
    (node: { id: string; data: { name?: string }; handles: object[] }) => ({
      ...node,
      type: "ping",
      style: {
        height: Math.max(80, node.handles.length * 20 + 20),
      },
      data: {
        handles: node.handles,
        ...node.data,
        scriptContent: journeyScripts?.[node.id] ?? [{}, { script: "" }],
        name: node.data.name?.startsWith("s")
          ? node.data.name
          : node.data.name === "70e691a5-1e33-4ac3-a356-e7b6d60d92e0"
            ? "Success"
            : node.data.name === "e301438c-0bd0-429c-ab0c-66126501069a"
              ? "Fail"
              : node.data.name,
      },
    }),
  );

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const edges = journeyFlow?.edges.map((e: any) => ({
    ...e,
    markerEnd: "arrow",
    type: "simplebezier",
    animated: e.style.stroke !== "grey",
    style: {
      ...e.style,
      strokeWidth: e.style.stroke !== "grey" ? 10 : undefined,
    },
  }));

  return (
    <Grid
      container
      spacing={1}
      style={{ display: "flex", flexDirection: "row" }}
    >
      <Grid size={3}>
        <FormControl fullWidth>
          <InputLabel id={"transaction-id"}>Select Transaction Id</InputLabel>
          <Select
            labelId={"transaction-id"}
            value={transactionId}
            onChange={(event) => setTransactionId(event.target.value)}
          >
            {[...(stateForJourney ?? []), ...(journeyTransactions ?? [])].map(
              (transactionId, i) => (
                <MenuItem key={i} value={transactionId}>
                  {transactionId}
                </MenuItem>
              ),
            )}
          </Select>
        </FormControl>
        <div style={{ padding: "30px" }}>
          {scriptLogs &&
            scriptLogs.result
              .filter(
                (res: unknown) =>
                  !JSON.stringify(res).includes("Unknown outcome"),
              )
              .map((log: { payload: Record<string, unknown> }, ix: number) => (
                <Log key={ix} data={log.payload} />
              ))}
        </div>
      </Grid>
      <Grid size={9}>
        <Grid container>
          <Grid size={12}>
            <Box component={"form"} sx={{ display: "flex", flexWrap: "wrap" }}>
              <FormControl fullWidth>
                <Controller
                  name="selectedTree"
                  control={control}
                  render={({ field }) => (
                    <>
                      <InputLabel id={"tree-select"}>Select Journey</InputLabel>
                      <Select {...field} labelId={"tree-select"}>
                        {((journeyList as string[]) ?? [])
                          .sort()
                          .map((name, i) => (
                            <MenuItem key={`tree-${i}`} value={name}>
                              {name}
                            </MenuItem>
                          ))}
                      </Select>
                    </>
                  )}
                />
              </FormControl>
              <Controller
                name={"startsWith"}
                control={control}
                render={({ field }) => (
                  <>
                    <TextField
                      label={"Starts With"}
                      id={"starts-with"}
                      {...field}
                    />
                  </>
                )}
              />
              <Controller
                name={"contains"}
                control={control}
                render={({ field }) => (
                  <>
                    <TextField label={"Contains"} id={"contains"} {...field} />
                  </>
                )}
              />
            </Box>
          </Grid>
          <Grid size={12} height={"80vh"}>
            <Box
              sx={{
                width: "100%",
                height: "100%",
              }}
            >
              {journeyScripts && (
                <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} />
              )}
            </Box>
          </Grid>
        </Grid>
      </Grid>
    </Grid>
  );
};

const jsonStringParser = <T extends z.ZodTypeAny>(schema: T) =>
  z
    .string()
    .transform((str, ctx) => {
      try {
        const cleanStr = str.replace(/"/g, "");
        switch (true) {
          case cleanStr.startsWith("joined"): {
            return { type: "system", message: "joined", msgText: cleanStr };
          }
          case cleanStr.startsWith("Total"): {
            return {
              type: "system",
              message: "Visitor count",
              msgText: cleanStr,
            };
          }
          case cleanStr.startsWith("Someone"): {
            return { type: "system", message: "joined", msgText: cleanStr };
          }
          case cleanStr.startsWith("pong"): {
            return { type: "system", message: "pong", msgText: cleanStr };
          }
          default: {
            return JSON.parse(str);
          }
        }
      } catch (e: unknown) {
        ctx.addIssue({
          code: "custom",
          message: `Invalid JSON string, or unknown message type: ${JSON.stringify(e)}`,
        });
        return z.NEVER;
      }
    })
    .pipe(schema);

const transactionId = z.object({
  type: z.literal("TransactionIdWs"),
  id: z.uuid(),
  journey: z.string(),
});

const systemMessage = z.object({
  type: z.literal("system"),
  message: z.string(),
  msgText: z.string(),
});

const webSocMessages = z.union([systemMessage, transactionId]);

const Flow = () => {
  const [state, dispatch] = useReducer(reducer, {
    flow: {},
    transactions: {},
  });
  const [wsState, setWsState] = useState<WebSocket>();

  useEffect(() => {
    const ws = new WebSocket("ws://localhost:8081/api/ws");
    setWsState(ws);
    ws.onopen = () => {
      console.info("connected to server");
    };
    ws.onmessage = (event) => {
      const data = jsonStringParser(webSocMessages).parse(event.data);

      switch (data.type) {
        case "TransactionIdWs": {
          dispatch(makeNewTransactionIdAction(data.journey, data.id));
          break;
        }
        default: {
          break;
        }
      }
    };
    ws.onclose = () => console.log("Disconnected");
    return () => ws.close();
  }, []);

  useEffect(() => {
    if (wsState !== undefined && wsState.readyState === WebSocket.OPEN) {
      try {
        wsState.send("/join all");
      } catch (e) {
        console.error(e);
      }
    }
  }, [wsState, wsState?.readyState]);

  return (
    <ReactFlowProvider>
      <FlowInner {...{ state, dispatch, ws: wsState }} />
    </ReactFlowProvider>
  );
};
export default Flow;
