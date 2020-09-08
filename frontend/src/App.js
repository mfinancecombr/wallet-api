import * as React from "react";
import { Admin, Resource, ListGuesser, fetchUtils } from "react-admin";
import jsonServerProvider from "ra-data-json-server";

import { BrokerList, BrokerEdit, BrokerCreate, BrokerIcon } from "./Brokers";
import {
  StockOperationCreate,
  StockOperationEdit,
  StockOperationIcon,
  StockOperationList,
} from "./StockOperations";

const dataProvider = jsonServerProvider("http://localhost:8000");
const App = () => (
  <Admin dataProvider={dataProvider}>
    <Resource
      name="brokers"
      list={BrokerList}
      edit={BrokerEdit}
      create={BrokerCreate}
    />
    <Resource
      name="stocks/operations"
      list={StockOperationList}
      edit={StockOperationEdit}
      create={StockOperationCreate}
    />
  </Admin>
);

export default App;
