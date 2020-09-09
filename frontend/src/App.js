import * as React from "react";
import { Admin, Resource } from "react-admin";
import jsonServerProvider from "ra-data-json-server";

import { BrokerList, BrokerEdit, BrokerCreate } from "./Brokers";
import { EventCreate, EventEdit, EventList } from "./Events";
import { PortfolioList, PortfolioEdit, PortfolioCreate } from "./Portfolios";

const dataProvider = jsonServerProvider("http://localhost:8000/api/v1");
const App = () => (
  <Admin dataProvider={dataProvider}>
    <Resource
      name="brokers"
      list={BrokerList}
      edit={BrokerEdit}
      create={BrokerCreate}
    />
    <Resource
      name="events"
      list={EventList}
      edit={EventEdit}
      create={EventCreate}
    />
    <Resource
      name="portfolios"
      list={PortfolioList}
      edit={PortfolioEdit}
      create={PortfolioCreate}
    />
  </Admin>
);

export default App;
