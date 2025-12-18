import { Handle, type NodeProps, Position, type Node } from "@xyflow/react";
import { useState } from "react";
import Modal from "@mui/material/Modal";
import Box from "@mui/material/Box";
import { ContextAwareButton } from "../../components";

export type PingNode = Node<
  {
    handles: Array<Handle>;
    name?: string;
    type?: string;
    scriptContent: [object, { script: string }];
  },
  "ping"
>;
const style = {
  position: "absolute",
  top: "50%",
  left: "50%",
  transform: "translate(-50%, -50%)",
  width: "80vw",
  height: "80vh",
  bgcolor: "background.paper",
  border: "2px solid #000",
  boxShadow: 24,
  p: 4,
  overflow: "scroll",
};

export const PingNode = ({ data }: NodeProps<PingNode>) => {
  const handles = data.handles as Array<Handle>;

  const [isOpen, setIsOpen] = useState(false);
  const handleOpen = () => setIsOpen(true);

  const handleClose = () => setIsOpen(false);

  const displayString = atob(data.scriptContent[1].script).split("\n");

  return (
    <>
      <Modal onClose={handleClose} open={isOpen}>
        <Box sx={style}>
          {displayString.map((line, i) => (
            <p
              style={{ marginTop: 0, marginBottom: 0, whiteSpace: "pre" }}
              key={i}
              id="modal-modal-description"
            >
              {line}
            </p>
          ))}
        </Box>
      </Modal>
      <Handle
        type={"target"}
        position={Position.Left}
        className="custom-handle"
        style={{ padding: 0 }}
      />
      <div className="resizer-node__handles">
        {handles.map((hand, i) => (
          <Handle
            type={"source"}
            position={Position.Right}
            style={{
              top: (i + 1) * 20,
              background: "none",
            }}
            id={hand.id}
            key={`${i}-${hand.id}-${hand.nodeId}`}
            className="resizer-node__handle custom-handle"
          >
            {hand.id}
          </Handle>
        ))}
      </div>
      <div>{data?.name ?? "ToDo"}</div>
      <div>{data?.type ?? "ToDo"}</div>
      <div style={{ position: "absolute", bottom: 0 }}>
        <ContextAwareButton
          style={{ margin: 0, padding: 0, fontSize: "x-small" }}
          onClick={handleOpen}
          vsCodeMessage={{
            event: "open",
            script: data?.name,
            content: displayString,
          }}
        >
          Script
        </ContextAwareButton>
      </div>
    </>
  );
};
