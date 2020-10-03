import * as React from "react";
import {
  Create,
  Datagrid,
  Edit,
  EditButton,
  List,
  ReferenceManyField,
  Show,
  ShowButton,
  SimpleForm,
  SimpleShowLayout,
  TextField,
  TextInput,
} from "react-admin";
import { Divider } from "@material-ui/core";
import { PositionDataGrid } from "./Positions";
import { Performance } from "./Performance";

import Allocation from "./Allocation";

const PortfolioShowTitle = ({ record }) => {
  return <span>Portfolio {record ? record.name : ""}</span>;
};

export const PortfolioShow = (props) => {
  return (
    <Show title={<PortfolioShowTitle />} {...props}>
      <SimpleShowLayout>
        <TextField source="name" />
        <Divider />
        <Performance portfolio={props.id} />
        <Divider />
        <ReferenceManyField
          reference="portfolios/positions"
          target="id"
          label="Charts"
        >
          <Allocation height={300} width={400} isMoney={true} />
        </ReferenceManyField>
        <ReferenceManyField
          reference="portfolios/positions"
          target="id"
          label="Positions"
        >
          <PositionDataGrid />
        </ReferenceManyField>
      </SimpleShowLayout>
    </Show>
  );
};

export const PortfolioList = (props) => (
  <List {...props}>
    <Datagrid>
      <TextField source="name" />
      <ShowButton basePath="/portfolios" />
      <EditButton basePath="/portfolios" />
    </Datagrid>
  </List>
);

const PortfolioName = ({ record }) => {
  return <span>Portfolio {record && record.name}</span>;
};

export const PortfolioEdit = (props) => (
  <Edit title={<PortfolioName />} {...props}>
    <SimpleForm>
      <TextInput disabled source="id" />
      <TextInput source="name" />
    </SimpleForm>
  </Edit>
);

export const PortfolioCreate = (props) => (
  <Create title="Create a Portfolio" {...props}>
    <SimpleForm>
      <TextInput source="name" />
    </SimpleForm>
  </Create>
);
