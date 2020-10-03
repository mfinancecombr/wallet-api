import * as React from "react";
import { Route } from "react-router-dom";
import { Admin, Resource } from "react-admin";
import jsonServerProvider from "ra-data-json-server";
import {
  AttachMoney as AttachMoneyIcon,
  FolderSpecial as FolderSpecialIcon,
  ShoppingCart as ShoppingCartIcon,
  Store as StoreIcon,
} from "@material-ui/icons";

import { BrokerList, BrokerEdit, BrokerCreate } from "./Brokers";
import { EventCreate, EventEdit, EventList } from "./Events";
import Menu from "./Menu";
import {
  PortfolioShow,
  PortfolioList,
  PortfolioEdit,
  PortfolioCreate,
} from "./Portfolios";
import { PositionList } from "./Positions";
import { Performance } from "./Performance";

const customRoutes = [
  <Route exact path="/performance" component={Performance} />,
];

const dataProvider = jsonServerProvider("http://localhost:8000/api/v1");
const App = () => (
  <Admin dataProvider={dataProvider} menu={Menu} customRoutes={customRoutes}>
    <Resource
      name="brokers"
      list={BrokerList}
      edit={BrokerEdit}
      create={BrokerCreate}
      icon={StoreIcon}
      options={{ label: "Brokers" }}
    />
    <Resource
      name="events"
      list={EventList}
      edit={EventEdit}
      create={EventCreate}
      icon={ShoppingCartIcon}
      options={{ label: "Events" }}
    />
    <Resource
      name="portfolios"
      show={PortfolioShow}
      list={PortfolioList}
      edit={PortfolioEdit}
      create={PortfolioCreate}
      icon={FolderSpecialIcon}
      options={{ label: "Portfolios" }}
    />
    <Resource
      name="positions"
      list={PositionList}
      icon={AttachMoneyIcon}
      options={{ label: "Positions" }}
    />

    <Resource name="portfolios/positions" />
    <Resource name="portfolios/performance" />
  </Admin>
);

export default App;
