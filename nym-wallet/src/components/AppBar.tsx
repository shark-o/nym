import React, { useContext, useEffect } from 'react'
import {
  AppBar as MuiAppBar,
  Divider,
  Grid,
  IconButton,
  Toolbar,
  Typography,
  useMediaQuery,
} from '@mui/material'
import { Box } from '@mui/system'
import { Logout } from '@mui/icons-material'
import { ClientContext } from '../context/main'
import { CopyToClipboard } from '.'

export const AppBar = () => {
  const { userBalance, logOut, clientDetails } = useContext(ClientContext)
  const matches = useMediaQuery('(min-width: 769px)')

  return (
    <MuiAppBar
      position="sticky"
      sx={{ boxShadow: 'none', bgcolor: 'nym.background.light' }}
    >
      <Toolbar>
        <Grid
          container
          justifyContent="space-between"
          alignItems="center"
          flexWrap="nowrap"
        >
          <Grid container item alignItems="center">
            <Grid item>
              <AppBarItem
                primaryText="Balance"
                secondaryText={userBalance.balance?.printable_balance}
              />
            </Grid>
            {matches && (
              <>
                <Divider
                  orientation="vertical"
                  variant="middle"
                  flexItem
                  sx={{ mr: 1 }}
                />

                <Grid item>
                  <AppBarItem
                    primaryText="Address"
                    secondaryText={clientDetails?.client_address}
                    Action={
                      <CopyToClipboard
                        text={clientDetails?.client_address}
                        iconButton
                      />
                    }
                  />
                </Grid>
              </>
            )}
          </Grid>
          <Grid item>
            <IconButton onClick={logOut} sx={{ color: 'nym.background.dark' }}>
              <Logout />
            </IconButton>
          </Grid>
        </Grid>
      </Toolbar>
    </MuiAppBar>
  )
}

const AppBarItem: React.FC<{
  primaryText: string
  secondaryText?: string
  Action?: React.ReactNode
}> = ({ primaryText, secondaryText = '', Action }) => {
  return (
    <Box sx={{ p: 1, mr: 1 }}>
      <Typography variant="body2" component="span" sx={{ color: 'grey.600' }}>
        {primaryText}:
      </Typography>{' '}
      <Typography
        variant="body2"
        component="span"
        color="nym.background.dark"
        sx={{ mr: 1 }}
      >
        {secondaryText}
      </Typography>
      {Action}
    </Box>
  )
}
