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

export const EventList = (props) => (
  <List {...props}>
    <Datagrid>
      <DateField showTime source="time" />
      <TextField source="symbol" />
      <TextField source="detail.price" label="Price" />
      <TextField source="detail.quantity" labe="Quantity" />
      <TextField source="detail.fees" label="Fees" />
      <TextField source="detail.type" label="Type" />
      <TextField source="detail.broker" label="Broker" />
      <ReferenceArrayField
        label="Portfolios"
        source="detail.portfolios"
        reference="portfolios"
      >
        <SingleFieldList>
          <ChipField source="name" />
        </SingleFieldList>
      </ReferenceArrayField>
      <EditButton basePath="/events" />
    </Datagrid>
  </List>
);

export const EventEdit = (props) => (
  <Edit title="Event operation" {...props}>
    <SimpleForm>
      <TextInput disabled source="id" />
      <DateTimeInput source="time" />
      <TextInput source="symbol" validate={required()} />
      <NumberInput label="Price" source="detail.price" validate={required()} />
      <NumberInput
        label="Quantity"
        source="detail.quantity"
        validate={required()}
      />
      <NumberInput label="Fees" source="detail.fees" />
      <SelectInput
        label="Price"
        source="detail.type"
        choices={[
          { id: "purchase", name: "Purchase" },
          { id: "sale", name: "Sale" },
        ]}
      />
      <TextInput label="Broker" source="detail.broker" />
      <ReferenceArrayInput
        label="Portfolios"
        source="detail.portfolios"
        reference="portfolios"
      >
        <SelectArrayInput source="detail.portfolios" />
      </ReferenceArrayInput>
    </SimpleForm>
  </Edit>
);

export const EventCreate = (props) => (
  <Create title="Create a stock operation" {...props}>
    <SimpleForm>
      <TextInput
        disabled
        source="eventType"
        validate={required()}
        defaultValue="Event"
      />
      <DateTimeInput source="time" />
      <TextInput source="symbol" validate={required()} />
      <NumberInput label="Price" source="detail.price" validate={required()} />
      <NumberInput
        label="Quantity"
        source="detail.quantity"
        validate={required()}
      />
      <NumberInput label="Fees" source="detail.fees" />
      <SelectInput
        label="Type"
        source="detail.type"
        choices={[
          { id: "purchase", name: "Purchase" },
          { id: "sale", name: "Sale" },
        ]}
        defaultValue="purchase"
      />
      <TextInput label="Broker" source="detail.broker" />
      <ReferenceArrayInput
        label="Portfolios"
        source="detail.portfolios"
        reference="portfolios"
      >
        <SelectArrayInput source="detail.portfolios" />
      </ReferenceArrayInput>
    </SimpleForm>
  </Create>
);
