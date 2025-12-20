import BluetoothSearching from "@mui/icons-material/BluetoothSearching";
import Search from "@mui/icons-material/Search";
import Box from "@mui/material/Box";
import Button from "@mui/material/Button";
import Divider from "@mui/material/Divider";
import Drawer from "@mui/material/Drawer";
import List from "@mui/material/List";
import ListItem from "@mui/material/ListItem";
import ListItemButton from "@mui/material/ListItemButton";
import ListItemIcon from "@mui/material/ListItemIcon";
import ListItemText from "@mui/material/ListItemText";
import "@xyflow/react/dist/style.css";
import { type ReactNode, useState } from "react";
import { Link, useSearchParams } from "react-router";
import { AppSharedContext } from "./Contexts.tsx";

const DrawerList = ({
  toggleDrawer,
}: {
  toggleDrawer: (input: boolean) => () => void;
}) => (
  <Box sx={{ width: 250 }} role="presentation" onClick={toggleDrawer(false)}>
    <List>
      {[["Flow", "/"]].map(([text, key], index) => (
        <ListItem key={text} disablePadding>
          <Link to={key}>
            <ListItemButton>
              <ListItemIcon>
                {index % 2 === 0 ? <Search /> : <BluetoothSearching />}
              </ListItemIcon>
              <ListItemText primary={text} />
            </ListItemButton>
          </Link>
        </ListItem>
      ))}
    </List>
    <Divider />
  </Box>
);

const AppNav = ({ children }: { children: ReactNode }) => {
  const [open, setOpen] = useState(false);

  const toggleDrawer = (newOpen: boolean) => () => {
    setOpen(newOpen);
  };
  const [urlSearchPrams] = useSearchParams();

  return (
    <AppSharedContext value={{ location: urlSearchPrams.get("view") ?? "web" }}>
      <div style={{ height: "90vh", width: "97vw" }}>
        <Button onClick={toggleDrawer(true)}>Open Side Menu</Button>
        <Drawer open={open} onClose={toggleDrawer(false)}>
          <DrawerList toggleDrawer={toggleDrawer} />
        </Drawer>
        <>{children}</>
        {/* Persistent bottom-left logo on every screen */}
        <img
          src={"/ade-logo.png"}
          alt="Agile Digital logo"
          style={{
            position: "fixed",
            left: 12,
            bottom: 12,
            width: 120,
            height: "auto",
            opacity: 0.9,
            pointerEvents: "none",
            zIndex: 9999,
          }}
        />
      </div>
    </AppSharedContext>
  );
};

export default AppNav;
