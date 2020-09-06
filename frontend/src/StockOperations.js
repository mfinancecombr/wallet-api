import * as React from "react";
import {
  List,
  Datagrid,
  Edit,
  Create,
  SimpleForm,
  ChipField,
  DateField,
  ReferenceArrayField,
  SingleFieldList,
  TextField,
  EditButton,
  TextInput,
  SelectInput,
  SelectArrayInput,
  ReferenceArrayInput,
  NumberInput,
} from "react-admin";
import { DateTimeInput } from "react-admin-date-inputs";
import { required } from "react-admin";

export const StockOperationList = (props) => (
  <List {...props}>
    <Datagrid>
      <TextField source="symbol" />
      <TextField source="price" />
      <TextField source="quantity" />
      <TextField source="fees" />
      <TextField source="type" />
      <TextField source="broker" />
      <ReferenceArrayField
        label="Portfolios"
        source="portfolios"
        reference="portfolios"
      >
        <SingleFieldList>
          <ChipField source="name" />
        </SingleFieldList>
      </ReferenceArrayField>
      <DateField showTime source="time" />
      <EditButton basePath="/stocks/operations" />
    </Datagrid>
  </List>
);

export const StockOperationEdit = (props) => (
  <Edit title="StockOperation operation" {...props}>
    <SimpleForm>
      <TextInput disabled source="id" />
      <TextInput source="symbol" validate={required()} />
      <NumberInput source="price" validate={required()} />
      <NumberInput source="quantity" validate={required()} />
      <NumberInput source="fees" />
      <SelectInput
        source="type"
        choices={[
          { id: "purchase", name: "Purchase" },
          { id: "sale", name: "Sale" },
        ]}
      />
      <TextInput source="broker" />
      <ReferenceArrayInput
        label="Portfolios"
        source="portfolios"
        reference="portfolios"
      >
        <SelectArrayInput source="portfolios" />
      </ReferenceArrayInput>
      <DateTimeInput source="time" />
    </SimpleForm>
  </Edit>
);

export const StockOperationCreate = (props) => (
  <Create title="Create a stock operation" {...props}>
    <SimpleForm initialValues={{ type: "purchase" }}>
      <TextInput source="symbol" validate={required()} />
      <NumberInput source="price" validate={required()} />
      <NumberInput source="quantity" validate={required()} />
      <NumberInput source="fees" />
      <SelectInput
        source="type"
        choices={[
          { id: "purchase", name: "Purchase" },
          { id: "sale", name: "Sale" },
        ]}
      />
      <TextInput source="broker" />
      <ReferenceArrayInput
        label="Portfolios"
        source="portfolios"
        reference="portfolios"
      >
        <SelectArrayInput source="portfolios" />
      </ReferenceArrayInput>
      <DateTimeInput source="time" />
    </SimpleForm>
  </Create>
);
