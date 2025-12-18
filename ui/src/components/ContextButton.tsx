import { useContext } from "react";
import { AppSharedContext } from "../Contexts.tsx";
import Button, { type ButtonProps } from "@mui/material/Button";

export interface ContextButton extends ButtonProps {
  vsCodeMessage: string | object;
}

const ContextButton = ({
  vsCodeMessage,
  children,
  ...props
}: ContextButton) => {
  const sharedContext = useContext(AppSharedContext);

  return (
    <Button
      {...props}
      onClick={
        sharedContext.location === "vscode"
          ? () => {
              window.parent.postMessage(vsCodeMessage, "*");
            }
          : props.onClick
      }
    >
      {children}
    </Button>
  );
};

export default ContextButton;
