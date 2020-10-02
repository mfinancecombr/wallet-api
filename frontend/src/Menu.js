import React, { createElement } from "react";
import { useSelector } from "react-redux";
import { useMediaQuery } from "@material-ui/core";
import { MenuItemLink, getResources } from "react-admin";
import { withRouter } from "react-router-dom";

const Menu = ({ onMenuClick, logout }) => {
  const isXSmall = useMediaQuery((theme) => theme.breakpoints.down("xs"));
  const open = useSelector((state) => state.admin.ui.sidebarOpen);
  const resources = useSelector(getResources);
  return (
    <div>
      {resources.map((resource) => {
        if (!resource.hasList) {
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
