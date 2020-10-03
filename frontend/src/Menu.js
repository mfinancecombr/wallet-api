import React, { createElement } from "react";
import { useSelector } from "react-redux";
import { Divider, useMediaQuery } from "@material-ui/core";
import { MenuItemLink, getResources } from "react-admin";
import { withRouter } from "react-router-dom";
import { AttachMoney as AttachMoneyIcon } from "@material-ui/icons";

const Menu = ({ onMenuClick, logout }) => {
  const isXSmall = useMediaQuery((theme) => theme.breakpoints.down("xs"));
  const open = useSelector((state) => state.admin.ui.sidebarOpen);
  const resources = useSelector(getResources);
  return (
    <div>
      <MenuItemLink
        key="performance"
        to="/performance"
        primaryText="Performance"
        leftIcon={createElement(AttachMoneyIcon)}
        onClick={onMenuClick}
        sidebarIsOpen={open}
      />
      <Divider />
      {resources.map((resource) => {
        if (resource.icon === undefined) {
          return null;
        }

        return (
          <MenuItemLink
            key={resource.name}
            to={`/${resource.name}`}
            primaryText={
              (resource.options && resource.options.label) || resource.name
            }
            leftIcon={createElement(resource.icon)}
            onClick={onMenuClick}
            sidebarIsOpen={open}
          />
        );
      })}
      {isXSmall && logout}
    </div>
  );
};

export default withRouter(Menu);
